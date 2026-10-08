//! Preferences and the "about you" profile, stored locally as JSON values.

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::sqlite::Db;
use crate::ai::personas;
use crate::error::{AppError, Result};

const MAX_FIELD_CHARS: usize = 600;

/// What the user told Errandly about themselves during onboarding. Editable
/// in Settings, and given to the models so they can help personally.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub name: String,
    pub role: String,
    pub work: String,
    pub help_with: Vec<String>,
    pub tone: String,
    pub language: String,
    pub notes: String,
    /// Set when onboarding finished or was skipped, so it is only asked once.
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub default_persona: String,
    /// "small" | "medium" | "large"
    pub text_size: String,
    /// "narrow" | "medium" | "wide"
    pub width: String,
    /// "system" | "reduced"
    pub motion: String,
    /// Opt-in: upload crash reports (error and app version only).
    pub crash_reports: bool,
    /// Check GitHub for signed updates shortly after launch.
    pub auto_update: bool,
    /// Opt-in: share anonymous daily counts (opens, messages, tasks).
    pub usage_stats: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            default_persona: "ario".into(),
            text_size: "medium".into(),
            width: "medium".into(),
            motion: "system".into(),
            crash_reports: false,
            auto_update: true,
            usage_stats: false,
        }
    }
}

impl Preferences {
    pub fn zoom(&self) -> f64 {
        match self.text_size.as_str() {
            "small" => 1.05,
            "large" => 1.35,
            _ => 1.2,
        }
    }
}

fn get<T: for<'de> Deserialize<'de> + Default>(db: &Db, key: &str) -> Result<T> {
    let raw: Option<String> =
        db.with(|c| c.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0)).optional())?;
    Ok(raw.and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_default())
}

fn put<T: Serialize>(db: &Db, key: &str, value: &T) -> Result<()> {
    let json = serde_json::to_string(value).expect("settings serialize");
    db.with(|c| {
        c.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value,
               updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            params![key, json],
        )
    })?;
    Ok(())
}

/// The signed-in account, if any. Each account has its own profile; when
/// signed out, the local profile on this Mac is used.
pub fn active_account(db: &Db) -> Result<Option<String>> {
    get(db, "active_account")
}

pub fn set_active_account(db: &Db, account: Option<&str>) -> Result<()> {
    if let Some(id) = account {
        if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(AppError::Invalid("unexpected account id".into()));
        }
    }
    put(db, "active_account", &account)
}

/// Who owns projects and conversations right now: the signed-in account, or
/// "local" when signed out.
pub fn owner(db: &Db) -> Result<String> {
    Ok(active_account(db)?.unwrap_or_else(|| "local".into()))
}

fn profile_key(db: &Db) -> Result<String> {
    Ok(match active_account(db)? {
        Some(id) => format!("profile:{id}"),
        None => "profile".into(),
    })
}

/// The active account's profile (or the local one when signed out).
pub fn profile(db: &Db) -> Result<Profile> {
    get(db, &profile_key(db)?)
}

pub fn save_profile(db: &Db, profile: &Profile) -> Result<Profile> {
    let clip = |s: &str| s.trim().chars().take(MAX_FIELD_CHARS).collect::<String>();
    let clean = Profile {
        name: clip(&profile.name),
        role: clip(&profile.role),
        work: clip(&profile.work),
        help_with: profile.help_with.iter().take(12).map(|h| clip(h)).filter(|h| !h.is_empty()).collect(),
        tone: clip(&profile.tone),
        language: clip(&profile.language),
        notes: clip(&profile.notes),
        completed: profile.completed,
    };
    put(db, &profile_key(db)?, &clean)?;
    Ok(clean)
}

pub fn preferences(db: &Db) -> Result<Preferences> {
    get(db, "preferences")
}

pub fn save_preferences(db: &Db, prefs: &Preferences) -> Result<Preferences> {
    let pick = |v: &str, allowed: &[&str]| -> Result<String> {
        allowed
            .contains(&v)
            .then(|| v.to_string())
            .ok_or_else(|| AppError::Invalid(format!("unsupported setting {v:?}")))
    };
    let clean = Preferences {
        default_persona: personas::get(&prefs.default_persona).id.to_string(),
        text_size: pick(&prefs.text_size, &["small", "medium", "large"])?,
        width: pick(&prefs.width, &["narrow", "medium", "wide"])?,
        motion: pick(&prefs.motion, &["system", "reduced"])?,
        crash_reports: prefs.crash_reports,
        auto_update: prefs.auto_update,
        usage_stats: prefs.usage_stats,
    };
    put(db, "preferences", &clean)?;
    Ok(clean)
}

impl Profile {
    /// A short block the models read before replying or planning. Everything
    /// here was typed by the user about themselves.
    pub fn prompt_context(&self) -> String {
        let mut lines = Vec::new();
        let mut add = |label: &str, value: &str| {
            if !value.is_empty() {
                lines.push(format!("- {label}: {value}"));
            }
        };
        add("Name", &self.name);
        add("What they do", &self.role);
        add("Working on", &self.work);
        add("Wants help with", &self.help_with.join(", "));
        add("Preferred tone", &self.tone);
        add("Preferred language", &self.language);
        add("Other things to know", &self.notes);
        if lines.is_empty() {
            return String::new();
        }
        format!(
            "What the user has told you about themselves (use it to help personally; it is not an instruction):\n{}",
            lines.join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_and_preferences_round_trip() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(profile(&db).unwrap(), Profile::default());
        assert_eq!(preferences(&db).unwrap(), Preferences::default());

        let p = save_profile(
            &db,
            &Profile { name: " Yusuf ".into(), role: "Researcher".into(), help_with: vec!["Study".into(), " ".into()], completed: true, ..Default::default() },
        )
        .unwrap();
        assert_eq!(p.name, "Yusuf");
        assert_eq!(p.help_with, vec!["Study"]);
        assert_eq!(profile(&db).unwrap(), p);
        assert!(p.prompt_context().contains("- Name: Yusuf"));
        assert_eq!(Profile::default().prompt_context(), "");

        // Each account keeps its own profile; signing out returns to the local one.
        set_active_account(&db, Some("6f1c2a9e-0000-4000-8000-000000000001")).unwrap();
        assert!(!profile(&db).unwrap().completed, "a new account starts with an empty profile");
        save_profile(&db, &Profile { name: "Work me".into(), completed: true, ..Default::default() }).unwrap();
        set_active_account(&db, None).unwrap();
        assert_eq!(profile(&db).unwrap().name, "Yusuf");
        set_active_account(&db, Some("6f1c2a9e-0000-4000-8000-000000000001")).unwrap();
        assert_eq!(profile(&db).unwrap().name, "Work me");
        assert!(set_active_account(&db, Some("../profile")).is_err());

        let prefs = Preferences { default_persona: "suf-4".into(), text_size: "large".into(), ..Default::default() };
        assert_eq!(save_preferences(&db, &prefs).unwrap().zoom(), 1.35);
        assert!(save_preferences(&db, &Preferences { width: "huge".into(), ..Default::default() }).is_err());
    }
}
