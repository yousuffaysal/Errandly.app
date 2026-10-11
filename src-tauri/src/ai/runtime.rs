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
        // A model runner left behind by a crash or force-quit still holds
        // gigabytes of memory; nothing else uses it, so end it first.
        if let Some(dir) = bin.parent() {
            reap_orphans(dir);
        }
        let log = crate::crash::log_dir().and_then(|dir| {
            std::fs::create_dir_all(&dir).ok()?;
            std::fs::File::create(dir.join("runtime.log")).ok()
        });
        let mut cmd = Command::new(&bin);
        // Its own process group, so quitting ends the model runners it
        // starts too, not just the server.
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
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

/// Stops the bundled runtime if Errandly started it, with the model
/// runners it started.
pub fn stop() {
    let Some(mut child) = CHILD.lock().unwrap_or_else(|e| e.into_inner()).take() else { return };
    let group = child.id() as libc::pid_t;
    // SAFETY: signals the process group Errandly created for the runtime.
    unsafe { libc::killpg(group, libc::SIGTERM) };
    for _ in 0..20 {
        if matches!(child.try_wait(), Ok(Some(_))) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    // SAFETY: as above; anything still running in the group is ended.
    unsafe { libc::killpg(group, libc::SIGKILL) };
    let _ = child.wait();
}

/// Ends model runners from `dir` whose parent has gone (adopted by launchd).
fn reap_orphans(dir: &Path) {
    let Ok(out) = Command::new("/bin/ps").args(["-axo", "pid=,ppid=,command="]).output() else { return };
    for (pid, _) in orphans(&String::from_utf8_lossy(&out.stdout), dir) {
        // SAFETY: a runner from Errandly's own runtime folder that no server owns.
        unsafe { libc::kill(pid, libc::SIGTERM) };
    }
}

/// (pid, command) of processes started from `dir` whose parent is launchd.
fn orphans(ps: &str, dir: &Path) -> Vec<(libc::pid_t, String)> {
    let prefix = dir.display().to_string() + "/";
    ps.lines()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let pid = parts.next()?.parse().ok()?;
            let ppid: u32 = parts.next()?.parse().ok()?;
            let cmd = parts.collect::<Vec<_>>().join(" ");
            (ppid == 1 && cmd.starts_with(&prefix)).then_some((pid, cmd))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_only_orphaned_runners_from_our_folder() {
        let dir = Path::new("/Applications/Errandly.app/Contents/Resources/runtime");
        let ps = "  740     1 /Applications/Errandly.app/Contents/Resources/runtime/llama-server --model x\n\
                   741   700 /Applications/Errandly.app/Contents/Resources/runtime/llama-server --model y\n\
                   742     1 /usr/local/bin/llama-server --model z\n\
                   743     1 /Applications/Errandly.app/Contents/Resources/runtime-old/ollama serve\n";
        let found: Vec<_> = orphans(ps, dir).into_iter().map(|(p, _)| p).collect();
        assert_eq!(found, [740]);
    }
}
