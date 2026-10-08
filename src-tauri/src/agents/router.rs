//! Decides what a chat message is asking for (AGENT-001, AGENT-002).

use serde_json::{json, Value};

use crate::ai::Llm;
use crate::error::Result;

const MAX_REPLY_CHARS: usize = 1200;

const SYSTEM: &str = "You are part of Errandly, an assistant that runs locally on the user's Mac. \
Classify the user's message into one intent:\n\
- organize_files: sorting, tidying, moving, grouping or cleaning up files or folders\n\
- summarize_documents: summaries, notes, study guides or questions from PDFs or documents\n\
- analyze_spreadsheet: spreadsheets, invoices, expenses, sales, totals or reports from data\n\
- chat: anything else\n\
For chat, write a short, friendly, helpful answer in \"reply\" (under 120 words). \
Be honest about what you can do today: organize the files in a folder the user adds to the conversation, \
always as a plan they approve first. Summarizing documents and building spreadsheet reports are coming soon. \
For the other intents leave \"reply\" empty. \
Never claim that you changed, read or opened anything on the user's computer.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    OrganizeFiles,
    SummarizeDocuments,
    AnalyzeSpreadsheet,
    Chat(String),
}

/// `voice` is the active model's personality, used for chat replies.
pub async fn route<L: Llm>(llm: &L, voice: &str, message: &str) -> Result<Intent> {
    let schema = json!({
        "type": "object",
        "properties": {
            "intent": {
                "type": "string",
                "enum": ["organize_files", "summarize_documents", "analyze_spreadsheet", "chat"]
            },
            "reply": { "type": "string" }
        },
        "required": ["intent", "reply"]
    });
    let raw = llm.chat_json(&format!("{voice}\n\n{SYSTEM}"), message, &schema).await?;
    Ok(parse(&raw))
}

fn parse(raw: &Value) -> Intent {
    match raw["intent"].as_str() {
        Some("organize_files") => Intent::OrganizeFiles,
        Some("summarize_documents") => Intent::SummarizeDocuments,
        Some("analyze_spreadsheet") => Intent::AnalyzeSpreadsheet,
        _ => {
            let reply: String = raw["reply"].as_str().unwrap_or("").trim().chars().take(MAX_REPLY_CHARS).collect();
            Intent::Chat(if reply.is_empty() {
                "I'm here. Right now I can organize the files in a folder you add to this conversation. \
                 Try “Sort this folder by file type.”"
                    .into()
            } else {
                reply
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_intents_and_falls_back_to_chat() {
        assert_eq!(parse(&json!({"intent": "organize_files", "reply": ""})), Intent::OrganizeFiles);
        assert_eq!(parse(&json!({"intent": "analyze_spreadsheet"})), Intent::AnalyzeSpreadsheet);
        assert_eq!(parse(&json!({"intent": "chat", "reply": " Hi! "})), Intent::Chat("Hi!".into()));
        assert!(matches!(parse(&json!({"intent": "delete_everything"})), Intent::Chat(r) if !r.is_empty()));
        assert!(matches!(parse(&json!("garbage")), Intent::Chat(_)));
    }
}
