pub mod ollama;

use std::future::Future;

use serde_json::Value;

use crate::error::Result;

/// A local model that answers with JSON constrained to a schema.
pub trait Llm: Send + Sync {
    fn chat_json(
        &self,
        system: &str,
        user: &str,
        schema: &Value,
    ) -> impl Future<Output = Result<Value>> + Send;
}
