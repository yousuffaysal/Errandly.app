//! The bundled local AI runtime (Ollama), so users don't need Homebrew (PRD §8.2).
//!
//! On launch: if an Ollama server is already answering on 127.0.0.1:11434 (for
//! example a developer's own), Errandly uses it. Otherwise it starts the copy
//! bundled inside the app, bound to the loopback interface only, using the
//! standard ~/.ollama model folder so models are shared and never downloaded
//! twice. The bundled server is stopped when Errandly quits.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Manager};

static CHILD: Mutex<Option<Child>> = Mutex::new(None);

/// Where the bundled `ollama` binary lives: inside the app's resources, or in
/// src-tauri/runtime during development.
pub fn bundled_binary(app: &AppHandle) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        candidates.push(res.join("runtime").join("ollama"));
    }
    if cfg!(debug_assertions) {
        candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime").join("ollama"));
    }
    candidates.into_iter().find(|p| p.is_file())
}

fn server_answers() -> bool {
    std::net::TcpStream::connect_timeout(&"127.0.0.1:11434".parse().expect("valid address"), Duration::from_millis(300)).is_ok()
}

/// Starts the bundled runtime unless a server is already running. Never blocks
/// app start-up; failures are logged and the UI shows "Local AI offline".
pub fn start(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        if server_answers() {
            return;
        }
        let Some(bin) = bundled_binary(&app) else {
            eprintln!("no bundled AI runtime found; expecting an Ollama server on 127.0.0.1:11434");
            return;
        };
        let log = crate::crash::log_dir().and_then(|dir| {
            std::fs::create_dir_all(&dir).ok()?;
            std::fs::File::create(dir.join("runtime.log")).ok()
        });
        let mut cmd = Command::new(&bin);
        cmd.arg("serve")
            .env("OLLAMA_HOST", "127.0.0.1:11434")
            .env("OLLAMA_FLASH_ATTENTION", "1")
            .env("OLLAMA_KV_CACHE_TYPE", "q8_0")
            .stdin(Stdio::null());
        if let Some(lib) = bin.parent() {
            // The release keeps its GPU libraries next to the binary.
            cmd.current_dir(lib);
        }
        match log {
            Some(f) => {
                let err = f.try_clone().map(Stdio::from).unwrap_or_else(|_| Stdio::null());
                cmd.stdout(Stdio::from(f)).stderr(err);
            }
            None => {
                cmd.stdout(Stdio::null()).stderr(Stdio::null());
            }
        }
        match cmd.spawn() {
            Ok(child) => *CHILD.lock().unwrap_or_else(|e| e.into_inner()) = Some(child),
            Err(e) => eprintln!("couldn't start the bundled AI runtime: {e}"),
        }
    });
}

/// Stops the bundled runtime if Errandly started it.
pub fn stop() {
    if let Some(mut child) = CHILD.lock().unwrap_or_else(|e| e.into_inner()).take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}
