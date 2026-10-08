//! Decides what a chat message is asking for (AGENT-001, AGENT-002).
//!
//! Obvious messages are routed instantly by keyword, so small talk never waits
//! for a model call. Anything ambiguous gets one short, schema-constrained
//! classification from the local model.

use serde_json::{json, Value};

use crate::ai::Llm;
use crate::error::Result;

const SYSTEM: &str = "You route messages for Errandly, an assistant that runs locally on the user's Mac. \
Classify the user's message into one intent:\n\
- organize_files: sorting, tidying, moving, grouping or cleaning up files or folders\n\
- summarize_documents: summaries, notes, study guides or questions from PDFs or documents\n\
- analyze_spreadsheet: spreadsheets, invoices, expenses, sales, totals or reports from data\n\
- chat: anything else, including greetings and questions\n\
Reply with JSON only.";

/// What every chat reply may honestly claim.
pub const CHAT_RULES: &str = "Keep replies short and friendly (under 120 words). \
Be honest about what you can do today: organize the files in a folder the user adds to the conversation \
with the + button, always as a plan they approve first. Summarizing documents and building spreadsheet \
reports are coming soon. Never claim that you changed, read or opened anything on the user's computer.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    OrganizeFiles,
    SummarizeDocuments,
    AnalyzeSpreadsheet,
    Chat,
}

pub async fn route<L: Llm>(llm: &L, message: &str) -> Result<Intent> {
    if let Some(intent) = quick(message) {
        return Ok(intent);
    }
    let schema = json!({
        "type": "object",
        "properties": {
            "intent": {
                "type": "string",
                "enum": ["organize_files", "summarize_documents", "analyze_spreadsheet", "chat"]
            }
        },
        "required": ["intent"]
    });
    Ok(parse(&llm.chat_json(SYSTEM, message, &schema).await?))
}

/// Keyword routing for clear-cut messages. `None` when it is ambiguous.
pub fn quick(message: &str) -> Option<Intent> {
    let text = message.to_lowercase();
    let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let has = |needles: &[&str]| needles.iter().any(|n| text.contains(n));

    let organize = has(&["organi", "sort ", "sort this", "tidy", "clean up", "cleanup", "declutter", "arrange", "put in order"]);
    let summarize = has(&["summar", "study notes", "study guide", "practice question", "key points", "tl;dr"]);
    let spreadsheet = has(&["spreadsheet", "excel", "xlsx", "invoice", "expense", "sales report", "budget", "revenue", "profit"]);
    match (organize, summarize, spreadsheet) {
        (true, false, false) => return Some(Intent::OrganizeFiles),
        (false, true, false) => return Some(Intent::SummarizeDocuments),
        (false, false, true) => return Some(Intent::AnalyzeSpreadsheet),
        (false, false, false) => {}
        _ => return None,
    }

    const SMALL_TALK: &[&str] = &[
        "hi", "hello", "hey", "hiya", "yo", "salam", "assalamualaikum", "thanks", "thank", "thx", "ok", "okay",
        "cool", "great", "nice", "morning", "evening", "bye", "goodbye", "who", "how", "what", "help",
    ];
    let small = words.len() <= 8 && words.first().is_some_and(|w| SMALL_TALK.contains(w));
    small.then_some(Intent::Chat)
}

fn parse(raw: &Value) -> Intent {
    match raw["intent"].as_str() {
        Some("organize_files") => Intent::OrganizeFiles,
        Some("summarize_documents") => Intent::SummarizeDocuments,
        Some("analyze_spreadsheet") => Intent::AnalyzeSpreadsheet,
        _ => Intent::Chat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_routes_clear_messages() {
        assert_eq!(quick("hi"), Some(Intent::Chat));
        assert_eq!(quick("Hello!"), Some(Intent::Chat));
        assert_eq!(quick("thanks a lot"), Some(Intent::Chat));
        assert_eq!(quick("Who are you?"), Some(Intent::Chat));
        assert_eq!(quick("Help me organize my Downloads folder."), Some(Intent::OrganizeFiles));
        assert_eq!(quick("Sort this folder by file type"), Some(Intent::OrganizeFiles));
        assert_eq!(quick("Help me summarize my research documents."), Some(Intent::SummarizeDocuments));
        assert_eq!(quick("Help me create an expense report."), Some(Intent::AnalyzeSpreadsheet));
        // Ambiguous or unrecognized: the model decides.
        assert_eq!(quick("Organize my invoices and summarize them"), None);
        assert_eq!(quick("Can you put the lecture slides somewhere sensible"), None);
    }

    #[test]
    fn parses_intents_and_falls_back_to_chat() {
        assert_eq!(parse(&json!({"intent": "organize_files"})), Intent::OrganizeFiles);
        assert_eq!(parse(&json!({"intent": "analyze_spreadsheet"})), Intent::AnalyzeSpreadsheet);
        assert_eq!(parse(&json!({"intent": "delete_everything"})), Intent::Chat);
        assert_eq!(parse(&json!("garbage")), Intent::Chat);
    }
}
