//! Sign-in session storage in the macOS Keychain (SEC-005). The webview's auth
//! client reads and writes its session through these commands instead of
//! browser storage. Only `sb-` keys (Supabase's prefix) are accepted.

use crate::error::{AppError, Result};

const SERVICE: &str = "studio.foxmen.errandly";

fn entry(key: &str) -> Result<keyring::Entry> {
    if !key.starts_with("sb-") || key.len() > 128 {
        return Err(AppError::Permission("unsupported keychain key".into()));
    }
    keyring::Entry::new(SERVICE, key).map_err(|e| AppError::Invalid(e.to_string()))
}

#[tauri::command]
pub fn secure_get(key: String) -> Result<Option<String>> {
    match entry(&key)?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Invalid(e.to_string())),
    }
}

#[tauri::command]
pub fn secure_set(key: String, value: String) -> Result<()> {
    entry(&key)?.set_password(&value).map_err(|e| AppError::Invalid(e.to_string()))
}

#[tauri::command]
pub fn secure_remove(key: String) -> Result<()> {
    match entry(&key)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Invalid(e.to_string())),
    }
}
