//! Client for an Ollama server on this Mac (the PRD's prototyping runtime, §8.2).
//! Requests only ever go to the loopback interface.

use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};

use super::personas::{self, Persona, BASE_MODEL, PERSONAS};
use super::Llm;
use crate::error::{AppError, Result};

pub const BASE_URL: &str = "http://127.0.0.1:11434";
/// Keep the model in memory between messages; reloading it costs ~30 s on an 8 GB Mac.
const KEEP_ALIVE: &str = "30m";
const CHAT_TEMPERATURE: f32 = 0.6;
/// Room for a full answer to a real question; small talk stays short by instruction.
const MAX_CHAT_TOKENS: u32 = 900;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PersonaStatus {
    #[serde(flatten)]
    pub persona: &'static Persona,
    pub installed: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub reachable: bool,
    /// The shared base weights are downloaded.
    pub base_installed: bool,
    pub personas: Vec<PersonaStatus>,
    /// The app ships its own runtime, so "offline" just means it's starting.
    pub runtime_bundled: bool,
}

#[derive(Clone)]
pub struct Ollama {
    client: reqwest::Client,
    model: String,
    temperature: f32,
}

fn client(timeout: Duration) -> reqwest::Client {
    ensure_tls_provider();
    reqwest::Client::builder()
        .timeout(timeout)
        .no_proxy()
        .build()
        .expect("static client config is valid")
}

/// The updater plugin builds the HTTP stack without a default TLS crypto
/// provider, and any client created without one panics. Install the `ring`
/// provider (already compiled in) once, before the first client.
pub fn ensure_tls_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn connect_error(e: reqwest::Error) -> AppError {
    if e.is_connect() {
        AppError::ai("connection refused")
    } else {
        AppError::ai(e.to_string())
    }
}

/// Names of models installed in Ollama, with the implicit `:latest` tag removed.
async fn installed_models() -> std::result::Result<Vec<String>, reqwest::Error> {
    // Generous: Ollama answers slowly while loading a model on an 8 GB Mac.
    let tags: Value = client(Duration::from_secs(10))
        .get(format!("{BASE_URL}/api/tags"))
        .send()
        .await?
        .json()
        .await?;
    Ok(tags["models"]
        .as_array()
        .map(|ms| {
            ms.iter()
                .filter_map(|m| m["name"].as_str())
                .map(|n| n.strip_suffix(":latest").unwrap_or(n).to_owned())
                .collect()
        })
        .unwrap_or_default())
}

pub async fn status(runtime_bundled: bool) -> AiStatus {
    let models = installed_models().await;
    let names = models.as_deref().unwrap_or_default();
    AiStatus {
        reachable: models.is_ok(),
        base_installed: names.iter().any(|n| n == BASE_MODEL),
        personas: PERSONAS
            .iter()
            .map(|p| PersonaStatus { persona: p, installed: names.contains(&p.model()) })
            .collect(),
        runtime_bundled,
    }
}

/// Where a reply stops being useful, if it has: a repeated line, a repeated
/// phrase, or a run-on stretch with no sentence ending. The caller keeps the
/// text before that point.
pub fn degenerate_from(text: &str) -> Option<usize> {
    let cut = repeated_from(text).or_else(|| repeated_phrase_from(text)).or_else(|| run_on_from(text))?;
    // Never end mid-sentence: back up to the last complete sentence or line.
    Some(text[..cut].rfind(['.', '!', '?', '\n']).map(|i| i + 1).unwrap_or(cut))
}

/// The start of the second occurrence of any 8-word phrase.
fn repeated_phrase_from(text: &str) -> Option<usize> {
    const N: usize = 8;
    let words: Vec<(usize, &str)> = text
        .split_whitespace()
        .map(|w| (w.as_ptr() as usize - text.as_ptr() as usize, w))
        .collect();
    let mut seen = std::collections::HashSet::new();
    for win in words.windows(N) {
        let key: Vec<String> = win.iter().map(|(_, w)| w.to_lowercase()).collect();
        if !seen.insert(key) {
            return Some(win[0].0);
        }
    }
    None
}

/// More than 500 characters since the last sentence ending or line break:
/// cut back to that last ending.
fn run_on_from(text: &str) -> Option<usize> {
    let last_end = text.rfind(['.', '!', '?', '\n', ':']).map(|i| i + 1).unwrap_or(0);
    (text.len() - last_end > 500).then_some(last_end)
}

/// If the latest finished paragraph (or list item) repeats an earlier one,
/// the byte offset where the repetition starts.
fn repeated_from(text: &str) -> Option<usize> {
    let norm = |s: &str| {
        s.trim()
            .trim_start_matches(|c: char| c.is_ascii_digit() || matches!(c, '.' | ')' | '-' | '*' | '•' | ' '))
            .replace("**", "")
            .to_lowercase()
    };
    // Only judge complete lines.
    let done = &text[..text.rfind('\n')?];
    let mut seen = std::collections::HashSet::new();
    let mut offset = 0;
    for line in done.split('\n') {
        let key = norm(line);
        if key.chars().count() >= 30 && !seen.insert(key) {
            return Some(offset);
        }
        offset += line.len() + 1;
    }
    None
}

/// Disk used by the shared base model. The four Errandly models reuse its
/// weights, so this is the whole cost.
pub async fn models_size() -> Option<u64> {
    let tags: Value = client(Duration::from_secs(5)).get(format!("{BASE_URL}/api/tags")).send().await.ok()?.json().await.ok()?;
    tags["models"]
        .as_array()?
        .iter()
        .find(|m| m["name"].as_str().map(|n| n.strip_suffix(":latest").unwrap_or(n)) == Some(BASE_MODEL))
        .and_then(|m| m["size"].as_u64())
}

/// Downloads the base weights (if needed) and creates the four persona models.
/// `progress` receives a 0–100 percentage and a short, user-facing status.
pub async fn install(progress: &(dyn Fn(u8, &str) + Sync)) -> Result<()> {
    let installed = installed_models().await.map_err(connect_error)?;
    if !installed.iter().any(|n| n == BASE_MODEL) {
        pull_base(progress).await?;
    }
    let http = client(Duration::from_secs(120));
    for (i, p) in PERSONAS.iter().enumerate() {
        progress(96 + i as u8, &format!("Teaching {}", p.name));
        let resp = http
            .post(format!("{BASE_URL}/api/create"))
            .json(&json!({
                "model": p.model(),
                "from": BASE_MODEL,
                "system": p.modelfile_system(),
                "parameters": { "temperature": p.temperature },
                "stream": false,
            }))
            .send()
            .await
            .map_err(connect_error)?;
        if !resp.status().is_success() {
            let body: Value = resp.json().await.unwrap_or_default();
            return Err(AppError::ai(format!(
                "couldn't create {}: {}",
                p.name,
                body["error"].as_str().unwrap_or("unknown error")
            )));
        }
    }
    progress(100, "Ready");
    Ok(())
}

async fn pull_base(progress: &(dyn Fn(u8, &str) + Sync)) -> Result<()> {
    let mut resp = client(Duration::from_secs(60 * 60))
        .post(format!("{BASE_URL}/api/pull"))
        .json(&json!({ "model": BASE_MODEL, "stream": true }))
        .send()
        .await
        .map_err(connect_error)?;
    // The response is newline-delimited JSON status objects.
    let mut buf = Vec::new();
    while let Some(chunk) = resp.chunk().await.map_err(|e| AppError::ai(e.to_string()))? {
        buf.extend_from_slice(&chunk);
        while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=nl).collect();
            let Ok(v) = serde_json::from_slice::<Value>(&line) else { continue };
            if let Some(err) = v["error"].as_str() {
                return Err(AppError::ai(format!("download failed: {err}")));
            }
            if let (Some(done), Some(total)) = (v["completed"].as_u64(), v["total"].as_u64()) {
                if total > 0 {
                    let pct = (done as f64 / total as f64 * 95.0) as u8;
                    progress(pct, &format!("Downloading ({:.1} of {:.1} GB)", gb(done), gb(total)));
                }
            } else if v["status"].as_str() == Some("verifying sha256 digest") {
                progress(95, "Verifying download");
            }
        }
    }
    Ok(())
}

fn gb(bytes: u64) -> f64 {
    bytes as f64 / 1e9
}

impl Ollama {
    pub fn for_persona(id: &str) -> Self {
        let p = personas::get(id);
        // Cold-loading a model on an 8 GB Mac can take a while.
        Self { client: client(Duration::from_secs(300)), model: p.model(), temperature: p.temperature }
    }

    /// Loads the model into memory ahead of the first message.
    pub async fn warm_up(&self) -> Result<()> {
        self.client
            .post(format!("{BASE_URL}/api/generate"))
            .json(&json!({ "model": self.model, "keep_alive": KEEP_ALIVE }))
            .send()
            .await
            .map_err(connect_error)?;
        Ok(())
    }

    /// A free-text chat reply, streamed: `on_text` receives the reply so far
    /// after every chunk. Stops early (keeping what was written) if `cancel` is set.
    pub async fn chat_stream(
        &self,
        system: &str,
        history: &[(&str, &str)],
        cancel: &std::sync::atomic::AtomicBool,
        on_text: &(dyn Fn(&str) + Sync),
    ) -> Result<String> {
        let mut messages = vec![json!({ "role": "system", "content": system })];
        messages.extend(history.iter().map(|(role, content)| json!({ "role": role, "content": content })));
        let mut resp = self
            .client
            .post(format!("{BASE_URL}/api/chat"))
            .json(&json!({
                "model": self.model,
                "stream": true,
                "keep_alive": KEEP_ALIVE,
                // Conversation needs natural sampling: greedy decoding makes
                // small models loop, and a strong repeat penalty makes them
                // ramble. Plans and numbers use precise() instead.
                "options": {
                    "temperature": CHAT_TEMPERATURE,
                    "top_p": 0.9,
                    "num_predict": MAX_CHAT_TOKENS,
                    "repeat_penalty": 1.05
                },
                "messages": messages,
            }))
            .send()
            .await
            .map_err(connect_error)?;
        if !resp.status().is_success() {
            let body: Value = resp.json().await.unwrap_or_default();
            return Err(AppError::ai(body["error"].as_str().unwrap_or("request failed")));
        }
        let (mut text, mut buf) = (String::new(), Vec::new());
        while let Some(chunk) = resp.chunk().await.map_err(|e| AppError::ai(e.to_string()))? {
            buf.extend_from_slice(&chunk);
            while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=nl).collect();
                let Ok(v) = serde_json::from_slice::<Value>(&line) else { continue };
                if let Some(err) = v["error"].as_str() {
                    return Err(AppError::ai(err));
                }
                if let Some(piece) = v["message"]["content"].as_str() {
                    text.push_str(piece);
                    // Small models sometimes start repeating themselves; stop
                    // at the first repeated paragraph and keep what came before.
                    if let Some(cut) = degenerate_from(&text) {
                        text.truncate(cut);
                        on_text(&text);
                        return Ok(text.trim().to_string());
                    }
                    on_text(&text);
                }
            }
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
        }
        Ok(text.trim().to_string())
    }

    /// The same model with sampling turned off, for plans that must be repeatable.
    pub fn precise(&self) -> Self {
        Self { temperature: 0.0, ..self.clone() }
    }
}

impl Llm for Ollama {
    async fn chat_json(&self, system: &str, user: &str, schema: &Value) -> Result<Value> {
        let body = json!({
            "model": self.model,
            "stream": false,
            "format": schema,
            "keep_alive": KEEP_ALIVE,
            "options": { "temperature": self.temperature },
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
        });
        let resp = self
            .client
            .post(format!("{BASE_URL}/api/chat"))
            .json(&body)
            .send()
            .await
            .map_err(connect_error)?;
        let status = resp.status();
        let payload: Value = resp.json().await.map_err(|e| AppError::ai(e.to_string()))?;
        if !status.is_success() {
            let msg = payload["error"].as_str().unwrap_or("request failed");
            return Err(AppError::ai(format!("{msg} (HTTP {status})")));
        }
        let content = payload["message"]["content"]
            .as_str()
            .ok_or_else(|| AppError::ai("empty response"))?;
        serde_json::from_str(content)
            .map_err(|e| AppError::ai(format!("model returned invalid JSON: {e}")))
    }
}

#[cfg(test)]
mod tests {
    /// Needs a running Ollama: `pnpm test:ollama`.
    #[test]
    #[ignore]
    fn live_status() {
        let s = tauri::async_runtime::block_on(super::status(false));
        println!("{s:#?}");
        assert!(s.reachable);
    }
}

#[cfg(test)]
mod live {
    /// Downloads the base (if missing) and creates the four models: `pnpm test:ollama`.
    #[test]
    #[ignore]
    fn live_install() {
        tauri::async_runtime::block_on(super::install(&|pct, label| println!("{pct:>3}% {label}"))).unwrap();
        let s = tauri::async_runtime::block_on(super::status(false));
        assert!(s.base_installed && s.personas.iter().all(|p| p.installed), "{s:#?}");
    }
}

#[cfg(test)]
mod tls {
    /// Regression: creating a client used to panic ("No rustls crypto provider")
    /// once the updater plugin was added. Only talks to a local server, if one is running.
    #[test]
    fn clients_can_be_created() {
        let s = tauri::async_runtime::block_on(super::status(false));
        let _ = s.reachable;
    }
}

#[cfg(test)]
mod repetition {
    use super::{degenerate_from, repeated_from};

    #[test]
    fn stops_at_the_first_repeated_paragraph() {
        let looped = "Tips:\n1. **Empty the Trash** to free space quickly.\n2. Remove large files you no longer need.\n3. **Empty the Trash** to free space quickly.\n";
        let cut = repeated_from(looped).unwrap();
        assert!(looped[..cut].ends_with("no longer need.\n"));
        assert_eq!(repeated_from("1. Short\n2. Short\n"), None, "short lines may repeat");
        assert_eq!(repeated_from("A unique first paragraph that is long enough.\nAnother different paragraph that is long.\n"), None);
    }

    #[test]
    fn stops_rambling_and_repeated_phrases() {
        let good = "Back up first. Then empty the Trash and remove old installers.";
        assert_eq!(degenerate_from(good), None);
        let salad = format!("Back up first. {}", "word ".repeat(150));
        assert_eq!(&salad[..degenerate_from(&salad).unwrap()], "Back up first.");
        let phrase = "Use Disk Utility to find the large files on your Mac today. Also use Disk Utility to find the large files on your Mac today.";
        let cut = degenerate_from(phrase).unwrap();
        assert!(phrase[..cut].trim_end().ends_with("on your Mac today."));
    }
}
