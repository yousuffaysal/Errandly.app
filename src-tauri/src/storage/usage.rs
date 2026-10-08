//! Opt-in, anonymous daily usage counts (PRD §31): how many times Errandly was
//! opened, messages sent and tasks completed per day, under a random install
//! id that is not linked to any account. Nothing is counted unless the user
//! turned on "Share anonymous usage statistics".

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::settings;
use super::sqlite::Db;
use crate::error::Result;

#[derive(Clone, Copy, Debug)]
pub enum Counter {
    Open,
    Message,
    Task,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DayCounts {
    pub day: String,
    pub opens: u32,
    pub messages: u32,
    pub tasks: u32,
}

/// A random id for this install, created on first use. Never tied to an account.
pub fn install_id(db: &Db) -> Result<String> {
    let existing: Option<String> =
        db.with(|c| c.query_row("SELECT value FROM settings WHERE key = 'install_id'", [], |r| r.get(0)).optional())?;
    if let Some(id) = existing.and_then(|v| serde_json::from_str::<String>(&v).ok()) {
        return Ok(id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    db.with(|c| {
        c.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('install_id', ?1)",
            [serde_json::to_string(&id).expect("string serializes")],
        )
    })?;
    Ok(id)
}

/// Adds one to today's counter, only if the user opted in.
pub fn bump(db: &Db, counter: Counter) -> Result<()> {
    if !settings::preferences(db)?.usage_stats {
        return Ok(());
    }
    db.with(|c| {
        let day: String = c.query_row("SELECT date('now')", [], |r| r.get(0))?;
        let key = format!("usage:{day}");
        let current: Option<String> =
            c.query_row("SELECT value FROM settings WHERE key = ?1", [&key], |r| r.get(0)).optional()?;
        let mut counts: DayCounts = current.and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_default();
        counts.day = day;
        match counter {
            Counter::Open => counts.opens += 1,
            Counter::Message => counts.messages += 1,
            Counter::Task => counts.tasks += 1,
        }
        c.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, serde_json::to_string(&counts).expect("counts serialize")],
        )?;
        Ok(())
    })
}

/// Days not yet fully sent, oldest first (at most 30).
pub fn pending(db: &Db) -> Result<Vec<DayCounts>> {
    db.with(|c| {
        c.prepare("SELECT value FROM settings WHERE key LIKE 'usage:%' ORDER BY key LIMIT 30")?
            .query_map([], |r| r.get::<_, String>(0))?
            .map(|v| v.map(|v| serde_json::from_str(&v).unwrap_or_default()))
            .collect()
    })
}

/// After a successful send: past days are done and removed; today keeps
/// counting (it is re-sent later with larger totals, which replace these).
pub fn mark_sent(db: &Db, day: &str) -> Result<()> {
    db.with(|c| {
        c.execute(
            "DELETE FROM settings WHERE key = ?1 AND ?2 < date('now')",
            params![format!("usage:{day}"), day],
        )
    })?;
    Ok(())
}

/// Forget everything counted so far (when the user turns sharing off).
pub fn clear(db: &Db) -> Result<()> {
    db.with(|c| c.execute("DELETE FROM settings WHERE key LIKE 'usage:%'", []))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::settings::Preferences;

    #[test]
    fn counts_only_when_opted_in() {
        let db = Db::open_in_memory().unwrap();
        bump(&db, Counter::Open).unwrap();
        assert!(pending(&db).unwrap().is_empty(), "off by default: nothing is counted");

        settings::save_preferences(&db, &Preferences { usage_stats: true, ..Default::default() }).unwrap();
        bump(&db, Counter::Open).unwrap();
        bump(&db, Counter::Message).unwrap();
        bump(&db, Counter::Message).unwrap();
        bump(&db, Counter::Task).unwrap();
        let days = pending(&db).unwrap();
        assert_eq!(days.len(), 1);
        assert_eq!((days[0].opens, days[0].messages, days[0].tasks), (1, 2, 1));

        mark_sent(&db, &days[0].day).unwrap();
        assert_eq!(pending(&db).unwrap().len(), 1, "today stays until the day is over");
        assert_eq!(install_id(&db).unwrap(), install_id(&db).unwrap(), "stable id");
        clear(&db).unwrap();
        assert!(pending(&db).unwrap().is_empty());
    }
}
