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
- chat: anything else, including greetings, questions, explanations, advice and writing\n\
Only choose organize_files, summarize_documents or analyze_spreadsheet when the user wants work done on their \
own files.\n\
Reply with JSON only.";

/// What every chat reply may honestly claim. Small models fill gaps with
/// plausible guesses, so the facts users ask about most are stated outright.
pub const CHAT_RULES: &str = "Be genuinely useful. Answer exactly what the user asked, and start with the answer itself. \
Match the length to the request: one or two sentences for small talk; for real questions, explanations, advice, \
plans or writing (emails, notes, summaries of what the user pasted), give a complete, well-organized answer with \
short paragraphs or lists. Do not greet the user or introduce yourself unless they ask who you are. \
Do not list these facts unless the question is about them. \
If asked what model, AI or technology you are, say you are your own name, one of Errandly's on-device models \
made for Errandly by Foxmen Studio, and that you can't share details about the technology behind it. \
Never name other AI models, model makers or AI software. \
These facts about Errandly are always true; never contradict them:\n\
- You run entirely on the user's Mac. You do NOT need the internet: after the one-time model download, \
everything works offline.\n\
- Files, chats and settings stay on this Mac. Nothing is uploaded, and nothing is sent to Foxmen Studio.\n\
- No account or subscription is needed. Signing in is optional.\n\
- Today you can organize the files in a folder the user adds to the conversation with the + button. You \
always show a plan first; nothing moves until the user approves it, and every change can be undone.\n\
- You can also summarize documents (PDF, Word, text, Markdown), answer questions about them, and analyze \
spreadsheets (Excel, CSV), in that folder or in files the user attaches with + then “Add files”. Spreadsheet \
numbers are always calculated exactly by Errandly, not guessed. Any answer can be saved as a PDF with its \
“Save as PDF” button.\n\
- You only know what the user has told you about themselves (shown below, if anything) and what they say \
in this conversation. If they haven't told you something, say you don't know it.\n\
- You cannot browse the web, send email, or see files the user hasn't added.\n\
If you are not sure about something, say so instead of guessing. \
Never claim that you changed, read or opened anything on the user's computer.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    OrganizeFiles,
    SummarizeDocuments,
    AnalyzeSpreadsheet,
    Chat,
}

/// Words that show the user means their actual files.
const FILE_WORDS: &[&str] = &[
    "folder", "file", "files", "downloads", "desktop", "pdf", "pdfs", "document", "documents", "docs", "spreadsheet",
    "spreadsheets", "excel", "csv", "xlsx", "sheet", "invoices", "receipts", "photos", "images", "pictures",
    "screenshots", "installers", "slides", "attachments",
];

/// Openings that make a message a question or a writing request, not a job.
const QUESTION_STARTS: &[&str] = &[
    "what", "how", "why", "when", "where", "which", "who", "should", "is ", "are ", "do ", "does ", "can you explain",
    "could you explain", "explain", "write", "draft", "compose", "translate", "rewrite", "tell me", "give me",
];

/// `has_folder`: a folder is attached to this conversation.
pub async fn route<L: Llm>(llm: &L, message: &str, has_folder: bool) -> Result<Intent> {
    if let Some(intent) = quick(message, has_folder) {
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

/// Keyword routing. A file job only starts when the user is clearly talking
/// about their files; everything else is a conversation. `None` means the
/// message is about files but which job is unclear, so the model decides.
pub fn quick(message: &str, has_folder: bool) -> Option<Intent> {
    let text = message.to_lowercase().replace('’', "'");
    let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let has = |needles: &[&str]| needles.iter().any(|n| text.contains(n));
    let mentions_files = words.iter().any(|w| FILE_WORDS.contains(w)) || has(&["this folder", "these files"]);
    let question = QUESTION_STARTS.iter().any(|q| text.trim_start().starts_with(q));
    // With a folder attached, an instruction ("tidy it up") is about that folder;
    // a question ("what should I clean up?") is still a conversation.
    if !mentions_files && !(has_folder && !question) {
        return Some(Intent::Chat);
    }

    let organize = has(&["organi", "renam", "sort ", "sort this", "sort my", "tidy", "clean up", "cleanup", "declutter", "arrange", "put in order", "group "]);
    let summarize = has(&["summar", "study notes", "study guide", "practice question", "key points", "tl;dr", "what's in", "read "]);
    let spreadsheet = has(&["spreadsheet", "excel", "xlsx", "csv", "invoice", "expense", "sales", "budget", "revenue", "profit", "total", "analy"]);
    match (organize, summarize, spreadsheet) {
        (true, false, false) => Some(Intent::OrganizeFiles),
        (false, true, false) => Some(Intent::SummarizeDocuments),
        (false, false, true) => Some(Intent::AnalyzeSpreadsheet),
        // About files, but no clear job (or several): let the model decide.
        _ if question && !organize && !summarize && !spreadsheet => Some(Intent::Chat),
        _ => None,
    }
}

/// True when the user is asking about the assistant itself, so an
/// introduction is the right answer.
pub fn asks_identity(message: &str) -> bool {
    let m = message.to_lowercase();
    ["who are you", "your name", "what are you", "introduce", "what model", "which model", "what ai", "which ai", "about yourself"]
        .iter()
        .any(|p| m.contains(p))
}

/// Removes an unrequested opening greeting or self-introduction ("Hi! I'm
/// Ario, Errandly's organizer.") so replies start with the answer.
/// Returns None while streaming if the opening sentence isn't finished yet.
pub fn strip_intro(text: &str, name: &str) -> Option<String> {
    let t = text.trim_start();
    let norm = |s: &str| s.to_lowercase().replace('’', "'");
    let starts_like = ["hi", "hello", "hey", "greetings", "i'm", "i am"].iter().any(|p| norm(t).starts_with(p));
    if !starts_like {
        return Some(text.to_string());
    }
    let name = name.to_lowercase();
    let mut rest = t;
    for _ in 0..2 {
        let Some(end) = rest.find(['.', '!', '?']) else {
            // The opening sentence is still arriving.
            return if rest.chars().count() < 200 { None } else { Some(rest.to_string()) };
        };
        let sentence = norm(&rest[..=end]);
        let greeting = sentence.split_whitespace().count() <= 3
            && ["hi", "hello", "hey", "greetings"].iter().any(|g| sentence.starts_with(g));
        let intro = (sentence.contains("i'm") || sentence.contains("i am"))
            && (sentence.contains(&name) || sentence.contains("errandly") || sentence.contains("assistant"));
        if !(greeting || intro) {
            break;
        }
        rest = rest[end + 1..].trim_start();
    }
    if rest.is_empty() { None } else { Some(rest.to_string()) }
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
        let q = |m: &str| quick(m, false);
        // Conversations stay conversations, even with "file job" words in them.
        for m in [
            "hi", "thanks a lot", "Who are you?", "Write a short polite email to my professor asking for an extension.",
            "Explain bayesian regression in simple words.", "My Mac is almost full. What should I clean up first?",
            "How do I make a monthly budget?", "Summarize the plot of Hamlet",
        ] {
            assert_eq!(q(m), Some(Intent::Chat), "{m}");
        }
        assert_eq!(q("Help me organize my Downloads folder."), Some(Intent::OrganizeFiles));
        assert_eq!(q("Sort this folder by file type"), Some(Intent::OrganizeFiles));
        assert_eq!(q("Summarize the documents in this folder."), Some(Intent::SummarizeDocuments));
        assert_eq!(q("Analyze the spreadsheets in this folder."), Some(Intent::AnalyzeSpreadsheet));
        assert_eq!(q("Which product sold most in sales.xlsx"), Some(Intent::AnalyzeSpreadsheet));
        // About files but unclear, or several jobs at once: the model decides.
        assert_eq!(q("Organize my invoices and summarize them"), None);
        assert_eq!(q("Can you put the lecture slides somewhere sensible"), None);
        // With a folder attached, a bare instruction is about that folder; a question isn't.
        assert_eq!(quick("tidy it up please", true), Some(Intent::OrganizeFiles));
        assert_eq!(quick("what should I clean up first?", true), Some(Intent::Chat));
        assert_eq!(quick("tidy it up please", false), Some(Intent::Chat));
    }

    #[test]
    fn unrequested_introductions_are_removed() {
        let s = |t: &str| strip_intro(t, "Ario");
        assert_eq!(s("Hi! I'm Ario, Errandly's organizer. Paris is the capital.").as_deref(), Some("Paris is the capital."));
        assert_eq!(s("I am Ario, your assistant. Yes, I work offline.").as_deref(), Some("Yes, I work offline."));
        assert_eq!(s("Hello! Here’s a plan: first, back up.").as_deref(), Some("Here’s a plan: first, back up."));
        assert_eq!(s("Paris is the capital of France.").as_deref(), Some("Paris is the capital of France."));
        assert_eq!(s("I'm not sure, but probably.").as_deref(), Some("I'm not sure, but probably."));
        assert_eq!(s("Hi! I'm Ar"), None, "wait for the sentence to finish");
        assert!(asks_identity("Who are you?") && asks_identity("which model do you use") && !asks_identity("hi"));
    }

    #[test]
    fn parses_intents_and_falls_back_to_chat() {
        assert_eq!(parse(&json!({"intent": "organize_files"})), Intent::OrganizeFiles);
        assert_eq!(parse(&json!({"intent": "analyze_spreadsheet"})), Intent::AnalyzeSpreadsheet);
        assert_eq!(parse(&json!({"intent": "delete_everything"})), Intent::Chat);
        assert_eq!(parse(&json!("garbage")), Intent::Chat);
    }
}
