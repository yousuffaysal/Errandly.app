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
const MAX_CHAT_TOKENS: u32 = 400;

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
}

#[derive(Clone)]
pub struct Ollama {
    client: reqwest::Client,
    model: String,
    temperature: f32,
}

fn client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .no_proxy()
        .build()
        .expect("static client config is valid")
}

fn connect_error(e: reqwest::Error) -> AppError {
    if e.is_connect() {
        AppError::Ai("the local AI service isn't running. Start it with `brew services start ollama`.".into())
    } else {
        AppError::Ai(e.to_string())
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

pub async fn status() -> AiStatus {
    let models = installed_models().await;
    let names = models.as_deref().unwrap_or_default();
    AiStatus {
        reachable: models.is_ok(),
        base_installed: names.iter().any(|n| n == BASE_MODEL),
        personas: PERSONAS
            .iter()
            .map(|p| PersonaStatus { persona: p, installed: names.contains(&p.model()) })
            .collect(),
    }
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
            return Err(AppError::Ai(format!(
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
    while let Some(chunk) = resp.chunk().await.map_err(|e| AppError::Ai(e.to_string()))? {
        buf.extend_from_slice(&chunk);
        while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=nl).collect();
            let Ok(v) = serde_json::from_slice::<Value>(&line) else { continue };
            if let Some(err) = v["error"].as_str() {
                return Err(AppError::Ai(format!("download failed: {err}")));
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
                "options": { "temperature": self.temperature, "num_predict": MAX_CHAT_TOKENS },
                "messages": messages,
            }))
            .send()
            .await
            .map_err(connect_error)?;
        if !resp.status().is_success() {
            let body: Value = resp.json().await.unwrap_or_default();
            return Err(AppError::Ai(body["error"].as_str().unwrap_or("request failed").to_string()));
        }
        let (mut text, mut buf) = (String::new(), Vec::new());
        while let Some(chunk) = resp.chunk().await.map_err(|e| AppError::Ai(e.to_string()))? {
            buf.extend_from_slice(&chunk);
            while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=nl).collect();
                let Ok(v) = serde_json::from_slice::<Value>(&line) else { continue };
                if let Some(err) = v["error"].as_str() {
                    return Err(AppError::Ai(err.to_string()));
                }
                if let Some(piece) = v["message"]["content"].as_str() {
                    text.push_str(piece);
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
        let payload: Value = resp.json().await.map_err(|e| AppError::Ai(e.to_string()))?;
        if !status.is_success() {
            let msg = payload["error"].as_str().unwrap_or("request failed");
            return Err(AppError::Ai(format!("{msg} (HTTP {status})")));
        }
        let content = payload["message"]["content"]
            .as_str()
            .ok_or_else(|| AppError::Ai("empty response from model".into()))?;
        serde_json::from_str(content)
            .map_err(|e| AppError::Ai(format!("model returned invalid JSON: {e}")))
    }
}

#[cfg(test)]
mod tests {
    /// Needs a running Ollama: `pnpm test:ollama`.
    #[test]
    #[ignore]
    fn live_status() {
        let s = tauri::async_runtime::block_on(super::status());
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
        let s = tauri::async_runtime::block_on(super::status());
        assert!(s.base_installed && s.personas.iter().all(|p| p.installed), "{s:#?}");
    }
}
