//! Crash reporting, privacy-first (PRD §24.2, §31.3).
//!
//! Every crash (a Rust panic, or a UI error reported by the webview) is written
//! to ~/Library/Logs/Errandly on this Mac. Nothing leaves the Mac unless the user
//! turns on "Send crash reports" in Settings; then the UI uploads unsent logs and
//! marks them sent. Reports hold the error and app version only: no chats, file
//! contents or profile, and the home folder path is replaced with "~".

use std::path::PathBuf;

use serde::Serialize;

use crate::error::{AppError, Result};

const KEEP: usize = 30;
const MAX_DETAIL: usize = 16_000;

pub fn log_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Logs/Errandly"))
}

/// Replaces the user's home folder with "~" so reports don't carry their username.
pub fn scrub(text: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() => text.replace(&home, "~"),
        _ => text.to_string(),
    }
}

fn write(kind: &str, detail: &str) -> Option<PathBuf> {
    let dir = log_dir()?;
    std::fs::create_dir_all(&dir).ok()?;
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_millis();
    let path = dir.join(format!("crash-{stamp}-{kind}.log"));
    let body: String = format!(
        "Errandly {} · macOS · {kind}\n\n{}",
        env!("CARGO_PKG_VERSION"),
        scrub(detail).chars().take(MAX_DETAIL).collect::<String>()
    );
    std::fs::write(&path, body).ok()?;
    prune(&dir);
    Some(path)
}

/// Keeps the newest logs only.
fn prune(dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut logs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("crash-")))
        .collect();
    logs.sort();
    let excess = logs.len().saturating_sub(KEEP);
    for old in &logs[..excess] {
        let _ = std::fs::remove_file(old);
    }
}

/// Records Rust panics before the default handler runs.
pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let bt = std::backtrace::Backtrace::force_capture();
        write("panic", &format!("{info}\n\n{bt}"));
        default(info);
    }));
}

#[tauri::command]
pub fn log_crash(kind: String, detail: String) -> Result<()> {
    let kind = if kind == "ui" { "ui" } else { "other" };
    write(kind, &detail).map(|_| ()).ok_or_else(|| AppError::Invalid("couldn't write the crash log".into()))
}

#[derive(Serialize)]
pub struct CrashReport {
    name: String,
    kind: String,
    detail: String,
}

/// Logs not uploaded yet (only used when the user opted in).
#[tauri::command]
pub fn unsent_crashes() -> Vec<CrashReport> {
    let Some(dir) = log_dir() else { return vec![] };
    let Ok(entries) = std::fs::read_dir(dir) else { return vec![] };
    let mut out: Vec<CrashReport> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            if !name.starts_with("crash-") || !name.ends_with(".log") || name.ends_with(".sent.log") {
                return None;
            }
            let kind = if name.contains("-panic") { "panic" } else { "ui" }.to_string();
            Some(CrashReport { detail: std::fs::read_to_string(e.path()).ok()?, name, kind })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out.truncate(10);
    out
}

#[tauri::command]
pub fn mark_crash_sent(name: String) -> Result<()> {
    let valid = name.starts_with("crash-")
        && name.ends_with(".log")
        && !name.ends_with(".sent.log")
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.');
    let dir = log_dir().filter(|_| valid).ok_or_else(|| AppError::Permission("unexpected log name".into()))?;
    let sent = name.trim_end_matches(".log").to_string() + ".sent.log";
    std::fs::rename(dir.join(&name), dir.join(sent))?;
    Ok(())
}

/// Shows the crash logs folder in Finder.
#[tauri::command]
pub fn open_crash_folder() -> Result<()> {
    let dir = log_dir().ok_or_else(|| AppError::Invalid("no home folder".into()))?;
    std::fs::create_dir_all(&dir)?;
    std::process::Command::new("open").arg(dir).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrubs_home_and_rejects_odd_names() {
        let home = std::env::var("HOME").unwrap();
        assert_eq!(scrub(&format!("at {home}/Desktop/x.rs")), "at ~/Desktop/x.rs");
        assert!(mark_crash_sent("../../etc/passwd".into()).is_err());
        assert!(mark_crash_sent("crash-1.sent.log".into()).is_err());
    }
}
