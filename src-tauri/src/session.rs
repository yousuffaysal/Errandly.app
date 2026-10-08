//! Where the sign-in session lives: Errandly's local database, readable only by
//! this macOS user (and encrypted by FileVault when it's on). The Keychain is
//! not used, so signing in never triggers a system password prompt.
//! Only `sb-` keys (Supabase's prefix) are accepted.

use rusqlite::{params, OptionalExtension};
use tauri::State;

use crate::commands::AppState;
use crate::error::{AppError, Result};

fn key(raw: &str) -> Result<String> {
    if !raw.starts_with("sb-") || raw.len() > 128 {
        return Err(AppError::Permission("unsupported session key".into()));
    }
    Ok(format!("auth:{raw}"))
}

#[tauri::command]
pub fn session_get(state: State<'_, AppState>, key: String) -> Result<Option<String>> {
    let k = self::key(&key)?;
    state.db.with(|c| c.query_row("SELECT value FROM settings WHERE key = ?1", [k], |r| r.get(0)).optional())
}

#[tauri::command]
pub fn session_set(state: State<'_, AppState>, key: String, value: String) -> Result<()> {
    let k = self::key(&key)?;
    state.db.with(|c| {
        c.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value,
               updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            params![k, value],
        )
    })?;
    Ok(())
}

#[tauri::command]
pub fn session_remove(state: State<'_, AppState>, key: String) -> Result<()> {
    let k = self::key(&key)?;
    state.db.with(|c| c.execute("DELETE FROM settings WHERE key = ?1", [k]))?;
    Ok(())
}
