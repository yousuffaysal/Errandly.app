//! Client for an Ollama server on this Mac (the PRD's prototyping runtime, §8.2).
//! Requests only ever go to the loopback interface.

use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};

use super::Llm;
use crate::error::{AppError, Result};

pub const BASE_URL: &str = "http://127.0.0.1:11434";
pub const DEFAULT_MODEL: &str = "qwen2.5:3b";

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub reachable: bool,
    pub models: Vec<String>,
    pub default_model: &'static str,
}

pub struct Ollama {
    client: reqwest::Client,
    model: String,
}

fn client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .no_proxy()
        .build()
        .expect("static client config is valid")
}

pub async fn status() -> AiStatus {
    let models = async {
        // Generous: Ollama answers slowly while loading a model on an 8 GB Mac.
        let tags: Value = client(Duration::from_secs(10))
            .get(format!("{BASE_URL}/api/tags"))
            .send()
            .await?
            .json()
            .await?;
        Ok::<_, reqwest::Error>(
            tags["models"]
                .as_array()
                .map(|ms| {
                    ms.iter()
                        .filter_map(|m| m["name"].as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
        )
    }
    .await;
    AiStatus {
        reachable: models.is_ok(),
        models: models.unwrap_or_default(),
        default_model: DEFAULT_MODEL,
    }
}

impl Ollama {
    pub fn new(model: impl Into<String>) -> Self {
        // Cold-loading a model on an 8 GB Mac can take a while.
        Self {
            client: client(Duration::from_secs(300)),
            model: model.into(),
        }
    }
}

impl Llm for Ollama {
    async fn chat_json(&self, system: &str, user: &str, schema: &Value) -> Result<Value> {
        let body = json!({
            "model": self.model,
            "stream": false,
            "format": schema,
            "options": { "temperature": 0 },
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
            .map_err(|e| {
                if e.is_connect() {
                    AppError::Ai("Ollama isn't running. Start it with `ollama serve`.".into())
                } else {
                    AppError::Ai(e.to_string())
                }
            })?;
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
    /// Needs a running Ollama: `cargo test -- --ignored live_status`.
    #[test]
    #[ignore]
    fn live_status() {
        let s = tauri::async_runtime::block_on(super::status());
        println!("{s:?}");
        assert!(s.reachable);
    }
}
