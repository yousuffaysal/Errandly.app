//! Google sign-in for a desktop app: the browser signs in with Google through
//! Supabase, which redirects to a one-shot listener on 127.0.0.1. The listener
//! hands the one-time code to the webview, which exchanges it (PKCE) for a session.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, Result};

/// Fixed so it can be listed in Supabase → Authentication → Redirect URLs.
pub const PORT: u16 = 53682;
pub const EVENT: &str = "errandly://oauth";
const WAIT: Duration = Duration::from_secs(300);

#[derive(Serialize, Clone)]
struct Callback {
    code: Option<String>,
    error: Option<String>,
}

/// Starts listening for the sign-in redirect and returns the port.
#[tauri::command]
pub fn start_oauth_listener(app: AppHandle) -> Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", PORT))
        .map_err(|_| AppError::Invalid("another sign-in is already in progress. Try again in a moment.".into()))?;
    std::thread::spawn(move || {
        listener.set_nonblocking(true).ok();
        let deadline = std::time::Instant::now() + WAIT;
        while std::time::Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false).ok();
                    let mut line = String::new();
                    let _ = BufReader::new(&stream).read_line(&mut line);
                    // "GET /auth/callback?code=...&state=... HTTP/1.1"
                    let target = line.split_whitespace().nth(1).unwrap_or("");
                    if !target.starts_with("/auth/callback") {
                        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
                        continue;
                    }
                    let callback = parse(target);
                    let ok = callback.code.is_some();
                    let _ = stream.write_all(page(ok).as_bytes());
                    let _ = app.emit(EVENT, callback);
                    return;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(150)),
                Err(_) => break,
            }
        }
        let _ = app.emit(EVENT, Callback { code: None, error: Some("sign-in timed out".into()) });
    });
    Ok(PORT)
}

/// Opens the provider's sign-in page in the default browser. Only Supabase
/// auth URLs are accepted, so the webview can't use this to open arbitrary links.
#[tauri::command]
pub fn open_auth_url(url: String) -> Result<()> {
    let host = url.strip_prefix("https://").and_then(|r| r.split('/').next()).unwrap_or("");
    // Builds with accounts configured only open their own project.
    let allowed = match option_env!("ERRANDLY_SUPABASE_HOST") {
        Some(own) => host == own,
        None => host.ends_with(".supabase.co"),
    };
    if !allowed || !url.contains("/auth/v1/authorize") {
        return Err(AppError::Permission("only Supabase sign-in pages can be opened".into()));
    }
    std::process::Command::new("open").arg(&url).spawn()?;
    Ok(())
}

fn parse(target: &str) -> Callback {
    let query = target.split_once('?').map(|(_, q)| q).unwrap_or("");
    let get = |key: &str| {
        query
            .split('&')
            .filter_map(|kv| kv.split_once('='))
            .find(|(k, _)| *k == key)
            .map(|(_, v)| decode(v))
    };
    Callback { code: get("code"), error: get("error_description").or_else(|| get("error")) }
}

fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(b);
                    i += 2;
                } else {
                    out.push(b'%');
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn page(ok: bool) -> String {
    let (title, body) = if ok {
        ("You’re signed in.", "You can close this tab and return to Errandly.")
    } else {
        ("Sign-in didn’t finish.", "Return to Errandly and try again.")
    };
    let html = format!(
        "<!doctype html><meta charset=utf-8><title>Errandly</title>\
         <body style=\"margin:0;display:grid;place-items:center;height:100vh;background:#fbfbf7;color:#1f241b;font-family:Georgia,serif;text-align:center\">\
         <div><div style=\"font-size:56px;color:#4a5a35\">✳</div><h1 style=\"font-weight:400\">{title}</h1>\
         <p style=\"font-family:Arial,sans-serif;color:#5f6655\">{body}</p></div>"
    );
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}", html.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_code_and_errors() {
        let c = parse("/auth/callback?code=abc%2D123&state=x");
        assert_eq!(c.code.as_deref(), Some("abc-123"));
        let e = parse("/auth/callback?error=access_denied&error_description=User+cancelled");
        assert_eq!((e.code, e.error.as_deref()), (None, Some("User cancelled")));
    }

    #[test]
    fn only_opens_supabase_auth_urls() {
        assert!(open_auth_url("https://evil.example.com/auth/v1/authorize".into()).is_err());
        assert!(open_auth_url("https://x.supabase.co/rest/v1/users".into()).is_err());
        assert!(open_auth_url("file:///etc/passwd".into()).is_err());
        assert!(open_auth_url("https://someone-else.supabase.co/auth/v1/authorize?provider=google".into()).is_err());
    }
}
