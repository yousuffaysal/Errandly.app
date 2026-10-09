//! Slash commands: "/organize", "/email …". A command says exactly what the
//! user wants, so nothing is guessed: file commands go straight to their agent,
//! and writing commands give the model a precise task.

use serde::Serialize;

use super::router::Intent;

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub name: &'static str,
    pub title: &'static str,
    pub hint: &'static str,
    /// Needs text after the command (e.g. what the email is about).
    pub takes_text: bool,
    /// Works on the conversation's folder.
    pub needs_folder: bool,
}

pub const COMMANDS: &[Command] = &[
    Command { name: "organize", title: "Organize folder", hint: "Sort the attached folder into clear folders", takes_text: false, needs_folder: true },
    Command { name: "rename", title: "Rename files", hint: "Give files clear, consistent names and sort them", takes_text: false, needs_folder: true },
    Command { name: "summarize", title: "Summarize documents", hint: "Summaries of the PDFs and documents in the folder", takes_text: false, needs_folder: true },
    Command { name: "analyze", title: "Analyze spreadsheet", hint: "Totals and insights from an Excel or CSV file", takes_text: false, needs_folder: true },
    Command { name: "email", title: "Write an email", hint: "A ready-to-send email: say who it's for and what about", takes_text: true, needs_folder: false },
    Command { name: "explain", title: "Explain simply", hint: "A clear explanation of any topic", takes_text: true, needs_folder: false },
    Command { name: "improve", title: "Improve writing", hint: "Rewrite your text so it's clearer and stronger", takes_text: true, needs_folder: false },
    Command { name: "translate", title: "Translate", hint: "Translate text, e.g. “/translate to Bangla: …”", takes_text: true, needs_folder: false },
    Command { name: "plan", title: "Make a plan", hint: "A step-by-step plan for any goal", takes_text: true, needs_folder: false },
    Command { name: "notes", title: "Study notes", hint: "Turn pasted text into clear notes with key points", takes_text: true, needs_folder: false },
];

/// What a slash command asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slash {
    /// A file job, with the request text the agent should see.
    Files(Intent, String),
    /// A writing job: the precise task to give the model.
    Write(String),
    /// "/something" that isn't a command.
    Unknown(String),
    /// A writing command with nothing to work on.
    MissingText(&'static Command),
}

pub fn parse(message: &str) -> Option<Slash> {
    let rest = message.trim_start().strip_prefix('/')?;
    let (name, text) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let name = name.to_lowercase();
    let text = text.trim();
    let Some(cmd) = COMMANDS.iter().find(|c| c.name == name) else {
        return Some(Slash::Unknown(name));
    };
    if cmd.takes_text && text.is_empty() {
        return Some(Slash::MissingText(cmd));
    }
    let extra = |t: &str| if t.is_empty() { String::new() } else { format!(" {t}") };
    Some(match cmd.name {
        "organize" => Slash::Files(Intent::OrganizeFiles, format!("organize this folder{}", extra(text))),
        "rename" => Slash::Files(Intent::OrganizeFiles, format!("rename the files with clear names and organize them{}", extra(text))),
        "summarize" => Slash::Files(Intent::SummarizeDocuments, format!("summarize the documents in this folder{}", extra(text))),
        "analyze" => Slash::Files(Intent::AnalyzeSpreadsheet, format!("analyze the spreadsheet{}", extra(text))),
        "email" => Slash::Write(format!(
            "Write a complete, ready-to-send email with a subject line, greeting, clear body and sign-off. \
             Use placeholders in [brackets] only for details I haven't given. The email: {text}"
        )),
        "explain" => Slash::Write(format!(
            "Explain this simply and clearly, with a short example or analogy, for someone new to it: {text}"
        )),
        "improve" => Slash::Write(format!(
            "Rewrite the following text so it is clearer, more natural and well-structured, keeping its meaning and \
             language. Reply with only the improved text:\n\n{text}"
        )),
        "translate" => Slash::Write(format!(
            "Translate the following as instructed. If no language is named, translate between English and Bangla. \
             Reply with only the translation:\n\n{text}"
        )),
        "plan" => Slash::Write(format!(
            "Make a practical, numbered step-by-step plan for this goal, with a rough time for each step: {text}"
        )),
        "notes" => Slash::Write(format!(
            "Turn the following into clear study notes: a one-line summary, then headings with short bullet points, \
             then 3 key takeaways:\n\n{text}"
        )),
        _ => unreachable!("every command is handled"),
    })
}

/// The reply for "/" followed by an unknown name.
pub fn help(unknown: &str) -> String {
    let list = COMMANDS.iter().map(|c| format!("/{} — {}", c.name, c.hint)).collect::<Vec<_>>().join("\n");
    format!("I don’t know “/{unknown}”. Here’s what you can use:\n\n{list}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands() {
        assert_eq!(parse("hello"), None);
        assert!(matches!(parse("/organize"), Some(Slash::Files(Intent::OrganizeFiles, _))));
        match parse("/rename") {
            Some(Slash::Files(Intent::OrganizeFiles, t)) => assert!(crate::agents::assets::wants_rename(&t)),
            other => panic!("{other:?}"),
        }
        assert!(matches!(parse("/Summarize"), Some(Slash::Files(Intent::SummarizeDocuments, _))));
        assert!(matches!(parse("/analyze sales.xlsx"), Some(Slash::Files(Intent::AnalyzeSpreadsheet, t)) if t.ends_with("sales.xlsx")));
        assert!(matches!(parse("/email ask my professor for 2 more days"), Some(Slash::Write(t)) if t.contains("professor")));
        assert!(matches!(parse("/email"), Some(Slash::MissingText(c)) if c.name == "email"));
        assert_eq!(parse("/dance"), Some(Slash::Unknown("dance".into())));
        assert!(help("dance").contains("/organize"));
    }
}
