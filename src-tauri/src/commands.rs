//! Tauri commands: the only entry points the UI has into the backend.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

use crate::agents::plan::Operation;
use crate::agents::planner::{self, Progress};
use crate::agents::router::{self, Intent};
use crate::agents::executor;
use crate::ai::ollama::{self, AiStatus, Ollama};
use crate::ai::personas::{self, Persona};
use crate::error::{AppError, Result};
use crate::security::permissions::canonical_root;
use crate::storage::conversations::{self, Conversation, Message, Role};
use crate::storage::projects::{self, Project};
use crate::storage::repo::{self, Grant, StepStatus, TaskStatus, TaskView};
use crate::storage::sqlite::Db;
use crate::tools::files::scan_folder;

const MAX_MESSAGE_CHARS: usize = 2000;
const MAX_INSTRUCTIONS_CHARS: usize = 2000;
pub const PROGRESS_EVENT: &str = "errandly://progress";
pub const REPLY_EVENT: &str = "errandly://reply";
/// How many recent messages a chat reply can see.
const CHAT_HISTORY: usize = 10;
const MAX_HISTORY_CHARS: usize = 1500;
pub const INSTALL_EVENT: &str = "errandly://install";

pub struct AppState {
    pub db: Arc<Db>,
    /// Cancellation flags for conversations that are thinking and tasks that are executing.
    running: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self { db: Arc::new(db), running: Mutex::default() }
    }

    fn start(&self, key: &str) -> Result<Arc<AtomicBool>> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if running.contains_key(key) {
            return Err(AppError::Invalid("Errandly is already working on this".into()));
        }
        let flag = Arc::new(AtomicBool::new(false));
        running.insert(key.to_string(), flag.clone());
        Ok(flag)
    }

    fn finish(&self, key: &str) {
        self.running.lock().unwrap_or_else(|e| e.into_inner()).remove(key);
    }

    fn signal(&self, key: &str) -> bool {
        let flag = self.running.lock().unwrap_or_else(|e| e.into_inner()).get(key).cloned();
        flag.map(|f| f.store(true, Ordering::Relaxed)).is_some()
    }

    fn is_running(&self, key: &str) -> bool {
        self.running.lock().unwrap_or_else(|e| e.into_inner()).contains_key(key)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationView {
    pub conversation: Conversation,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ReplyEvent {
    conversation_id: String,
    text: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    conversation_id: String,
    stage: u8,
    label: String,
}

fn conversation_view(db: &Db, id: &str) -> Result<ConversationView> {
    Ok(ConversationView { conversation: conversations::get(db, id)?, messages: conversations::messages(db, id)? })
}

// ---- local AI -------------------------------------------------------------

#[tauri::command]
pub async fn ai_status() -> AiStatus {
    ollama::status().await
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct InstallEvent {
    percent: u8,
    label: String,
}

/// Downloads the shared base model and creates Errandly's four models, with
/// progress events for the UI. Safe to run again; existing models are refreshed.
#[tauri::command]
pub async fn install_models(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    state.start("install")?;
    let emit = |percent: u8, label: &str| {
        let _ = app.emit(INSTALL_EVENT, InstallEvent { percent, label: label.to_string() });
    };
    let result = ollama::install(&emit).await;
    state.finish("install");
    result
}

/// Loads a model into memory so the first message doesn't wait for it.
#[tauri::command]
pub async fn warm_up(persona: String) -> Result<()> {
    Ollama::for_persona(&persona).warm_up().await
}

// ---- projects -------------------------------------------------------------

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>> {
    projects::list(&state.db)
}

#[tauri::command]
pub fn create_project(state: State<'_, AppState>, name: String, description: String) -> Result<Project> {
    projects::create(&state.db, &name, &description)
}

#[tauri::command]
pub fn update_project(state: State<'_, AppState>, project_id: String, name: String, description: String) -> Result<Project> {
    projects::update(&state.db, &project_id, &name, &description)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, project_id: String) -> Result<()> {
    projects::delete(&state.db, &project_id)
}

// ---- conversations --------------------------------------------------------

#[tauri::command]
pub fn list_conversations(state: State<'_, AppState>, project_id: String) -> Result<Vec<Conversation>> {
    conversations::list(&state.db, &project_id)
}

#[tauri::command]
pub fn create_conversation(state: State<'_, AppState>, project_id: String, persona: String) -> Result<ConversationView> {
    let conv = conversations::create(&state.db, &project_id, personas::get(&persona).id)?;
    conversation_view(&state.db, &conv.id)
}

#[tauri::command]
pub fn rename_conversation(state: State<'_, AppState>, conversation_id: String, title: String) -> Result<Conversation> {
    conversations::rename(&state.db, &conversation_id, &title)?;
    conversations::get(&state.db, &conversation_id)
}

#[tauri::command]
pub fn delete_conversation(state: State<'_, AppState>, conversation_id: String) -> Result<()> {
    if state.is_running(&conversation_id) {
        return Err(AppError::Invalid("wait for Errandly to finish before deleting this conversation".into()));
    }
    conversations::delete(&state.db, &conversation_id)
}

#[tauri::command]
pub fn set_persona(state: State<'_, AppState>, conversation_id: String, persona: String) -> Result<Conversation> {
    conversations::set_persona(&state.db, &conversation_id, personas::get(&persona).id)?;
    conversations::get(&state.db, &conversation_id)
}

#[tauri::command]
pub fn get_conversation(state: State<'_, AppState>, conversation_id: String) -> Result<ConversationView> {
    conversation_view(&state.db, &conversation_id)
}

#[tauri::command]
pub fn set_instructions(state: State<'_, AppState>, conversation_id: String, instructions: String) -> Result<()> {
    if instructions.chars().count() > MAX_INSTRUCTIONS_CHARS {
        return Err(AppError::Invalid(format!("keep instructions under {MAX_INSTRUCTIONS_CHARS} characters")));
    }
    conversations::set_instructions(&state.db, &conversation_id, &instructions)
}

/// Opens the native folder picker and attaches the chosen folder to the
/// conversation. Only folders chosen here are ever granted.
#[tauri::command]
pub async fn attach_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<ConversationView> {
    if let Some(grant) = pick_and_grant(app, &state.db).await? {
        conversations::set_grant(&state.db, &conversation_id, Some(&grant.id))?;
    }
    conversation_view(&state.db, &conversation_id)
}

/// Detaches the folder from this conversation and revokes the grant if no
/// other conversation still uses it.
#[tauri::command]
pub fn detach_folder(state: State<'_, AppState>, conversation_id: String) -> Result<ConversationView> {
    let conv = conversations::get(&state.db, &conversation_id)?;
    conversations::set_grant(&state.db, &conversation_id, None)?;
    if let Some(grant_id) = conv.grant_id {
        if !conversations::grant_in_use(&state.db, &grant_id)? {
            repo::delete_grant(&state.db, &grant_id)?;
            repo::audit(&state.db, None, "revoke", conv.folder.as_deref().unwrap_or(""))?;
        }
    }
    conversation_view(&state.db, &conversation_id)
}

async fn pick_and_grant(app: AppHandle, db: &Db) -> Result<Option<Grant>> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().set_title("Choose a folder Errandly may organize").blocking_pick_folder()
    })
    .await
    .map_err(|e| AppError::Invalid(e.to_string()))?;
    let Some(picked) = picked else { return Ok(None) };
    let path = picked.into_path().map_err(|e| AppError::Invalid(e.to_string()))?;
    let root = canonical_root(&path)?;

    let home = std::env::var_os("HOME").map(PathBuf::from);
    if root.parent().is_none() || Some(&root) == home.as_ref() {
        return Err(AppError::Permission(
            "choose a specific folder rather than your whole disk or home folder".into(),
        ));
    }
    let grant = repo::upsert_grant(db, &root.display().to_string())?;
    repo::audit(db, None, "grant", &grant.path)?;
    Ok(Some(grant))
}

/// Saves a user message, works out what it asks for, and answers. For an
/// organize request this plans (without touching files) and attaches the plan
/// to the reply for approval.
#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    text: String,
) -> Result<ConversationView> {
    let text = text.trim().to_string();
    if text.is_empty() || text.chars().count() > MAX_MESSAGE_CHARS {
        return Err(AppError::Invalid(format!("keep messages between 1 and {MAX_MESSAGE_CHARS} characters")));
    }
    let conv = conversations::get(&state.db, &conversation_id)?;
    let cancel = state.start(&conversation_id)?;
    conversations::add_message(&state.db, &conversation_id, Role::User, &text, None)?;

    let emit = |stage: u8, label: String| {
        let _ = app.emit(PROGRESS_EVENT, ProgressEvent { conversation_id: conversation_id.clone(), stage, label });
    };
    let persona = personas::get(&conv.persona);
    let reply = |text: &str| {
        let _ = app.emit(REPLY_EVENT, ReplyEvent { conversation_id: conversation_id.clone(), text: text.to_string() });
    };
    let llm = Ollama::for_persona(persona.id);
    let result = respond(&state.db, &conv, &text, &llm, persona, &cancel, &emit, &reply).await;
    state.finish(&conversation_id);

    let (reply, task_id) = match result {
        Ok(r) => r,
        Err(AppError::Cancelled) => ("Stopped. Nothing was changed.".to_string(), None),
        Err(e) => (format!("I couldn’t finish that: {e}"), None),
    };
    conversations::add_message(&state.db, &conversation_id, Role::Assistant, &reply, task_id.as_deref())?;
    conversation_view(&state.db, &conversation_id)
}

async fn respond(
    db: &Db,
    conv: &Conversation,
    text: &str,
    llm: &Ollama,
    persona: &Persona,
    cancel: &AtomicBool,
    emit: &(dyn Fn(u8, String) + Sync),
    on_reply: &(dyn Fn(&str) + Sync),
) -> Result<(String, Option<String>)> {
    emit(0, "Understanding your request".into());
    let intent = router::route(&llm.precise(), text).await?;
    if cancel.load(Ordering::Relaxed) {
        return Err(AppError::Cancelled);
    }

    let next_phase = "is coming in the next phase of Errandly. Right now I can organize the files in a folder \
                      you add to this conversation. Try “Sort this folder by file type.”";
    match intent {
        Intent::Chat => {
            // Recent messages, oldest first; the current message is already saved.
            let all = conversations::messages(db, &conv.id)?;
            let recent: Vec<(String, String)> = all
                .iter()
                .skip(all.len().saturating_sub(CHAT_HISTORY))
                .map(|m| (m.role.clone(), m.text.chars().take(MAX_HISTORY_CHARS).collect()))
                .collect();
            let history: Vec<(&str, &str)> = recent.iter().map(|(r, t)| (r.as_str(), t.as_str())).collect();
            let system = format!("{} You run locally on the user's Mac as part of Errandly. {}", persona.voice, router::CHAT_RULES);
            let reply = llm.chat_stream(&system, &history, cancel, on_reply).await?;
            if reply.is_empty() {
                return Err(if cancel.load(Ordering::Relaxed) { AppError::Cancelled } else { AppError::Ai("the model gave an empty reply".into()) });
            }
            Ok((reply, None))
        }
        Intent::SummarizeDocuments => Ok((format!("Turning documents into notes {next_phase}"), None)),
        Intent::AnalyzeSpreadsheet => Ok((format!("Building reports from spreadsheets {next_phase}"), None)),
        Intent::OrganizeFiles => {
            let (Some(grant_id), Some(folder)) = (&conv.grant_id, &conv.folder) else {
                return Ok((
                    "Happy to bring a little order. Which folder should I work in? Add one with the + button \
                     below. I can only see folders you choose."
                        .into(),
                    None,
                ));
            };
            let _ = grant_id;
            let root = PathBuf::from(folder);
            if canonical_root(&root)? != root {
                return Err(AppError::Permission("that folder has moved; add it to this conversation again".into()));
            }
            let files = scan_folder(&root)?;
            let instruction = if conv.instructions.trim().is_empty() {
                text.to_string()
            } else {
                format!("{text}\n\nThe user's standing preferences for this conversation: {}", conv.instructions.trim())
            };

            let task_id = repo::create_task(db, text, persona.name, folder, Some(&conv.id))?;
            let progress = |p: Progress| match p {
                Progress::ChoosingFolders => emit(1, "Choosing the right folders".into()),
                Progress::Sorting { done, total } => emit(2, format!("Sorting files ({done} of {total})")),
            };
            match planner::plan_organize(&llm.precise(), &instruction, &root, &files, persona.organize_style, cancel, &progress).await {
                Ok(p) => {
                    let meta = serde_json::to_string(&p.meta).expect("plan meta serializes");
                    repo::save_plan(db, &task_id, &meta, &p.operations)?;
                    let moves = p.operations.iter().filter(|o| matches!(o, Operation::MoveFile { .. })).count();
                    let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    if moves == 0 {
                        repo::finish_task(db, &task_id, TaskStatus::Cancelled, Some("nothing to move"))?;
                        return Ok((
                            format!(
                                "I looked through {name}, but couldn’t confidently place any of its files, so there’s \
                                 nothing to approve. Try naming the folders you want, like “Sort into Documents, \
                                 Images and Installers.”"
                            ),
                            None,
                        ));
                    }
                    let folders = p.operations.iter().filter_map(|o| match o {
                        Operation::MoveFile { to, .. } => to.parent(),
                        _ => None,
                    });
                    let folder_count = folders.collect::<std::collections::HashSet<_>>().len();
                    Ok((
                        format!(
                            "Here’s my plan for {name}. I’d sort {moves} of {} files into {folder_count} folders. \
                             Nothing moves until you approve.",
                            p.meta.scanned_files
                        ),
                        Some(task_id),
                    ))
                }
                Err(e) => {
                    let status = if matches!(e, AppError::Cancelled) { TaskStatus::Cancelled } else { TaskStatus::Failed };
                    repo::finish_task(db, &task_id, status, Some(&e.to_string()))?;
                    Err(e)
                }
            }
        }
    }
}

/// Stops whatever this conversation is thinking about, after the current model call.
#[tauri::command]
pub fn stop_conversation(state: State<'_, AppState>, conversation_id: String) -> bool {
    state.signal(&conversation_id)
}

#[tauri::command]
pub async fn export_conversation(app: AppHandle, state: State<'_, AppState>, conversation_id: String) -> Result<bool> {
    let view = conversation_view(&state.db, &conversation_id)?;
    let body = view
        .messages
        .iter()
        .map(|m| format!("{}\n{}", if m.role == "user" { "You" } else { "Errandly" }, m.text))
        .collect::<Vec<_>>()
        .join("\n\n");
    let name = format!("{}.txt", view.conversation.title.replace(['/', ':'], "-"));
    let target = tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().set_file_name(name).add_filter("Text", &["txt"]).blocking_save_file()
    })
    .await
    .map_err(|e| AppError::Invalid(e.to_string()))?;
    let Some(target) = target else { return Ok(false) };
    let path = target.into_path().map_err(|e| AppError::Invalid(e.to_string()))?;
    std::fs::write(path, body)?;
    Ok(true)
}

// ---- tasks ----------------------------------------------------------------

/// The user approved the plan shown to them: run it, then report back in the conversation.
#[tauri::command]
pub async fn approve_task(state: State<'_, AppState>, task_id: String) -> Result<TaskView> {
    let cancel = state.start(&task_id)?;
    let db = state.db.clone();
    let id = task_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || executor::execute(&db, &id, &cancel))
        .await
        .map_err(|e| AppError::Invalid(e.to_string()));
    state.finish(&task_id);
    let outcome = result.and_then(|r| r);
    let task = repo::get_task(&state.db, &task_id)?;

    let count = |status: StepStatus| {
        task.steps.iter().filter(|s| s.status == status && matches!(s.op, Operation::MoveFile { .. })).count()
    };
    let note = match (&outcome, task.status) {
        (Err(e), _) => format!("That didn’t work, and no files were moved: {e}"),
        (Ok(_), TaskStatus::Completed) => format!(
            "Done. I moved {} files and checked each one on disk. You can undo this anytime.",
            count(StepStatus::Done)
        ),
        (Ok(_), TaskStatus::Cancelled) => "Stopped before anything moved. Nothing was changed.".into(),
        (Ok(_), TaskStatus::Failed) => format!(
            "That didn’t work, and no files were moved{}",
            task.error.as_deref().map(|e| format!(": {e}")).unwrap_or_else(|| ".".into())
        ),
        (Ok(_), _) => format!(
            "I finished part of the plan: {} moved, {} failed, {} not started. Everything that moved was checked \
             on disk, and you can undo it.",
            count(StepStatus::Done),
            count(StepStatus::Failed),
            count(StepStatus::Skipped)
        ),
    };
    if let Some(conv) = &task.conversation_id {
        conversations::add_message(&state.db, conv, Role::Assistant, &note, None)?;
    }
    outcome?;
    Ok(task)
}

/// Rejects a plan awaiting approval, or asks a running task to stop after its current step.
#[tauri::command]
pub fn cancel_task(state: State<'_, AppState>, task_id: String) -> Result<TaskView> {
    if !state.signal(&task_id) {
        let task = repo::get_task(&state.db, &task_id)?;
        if task.status == TaskStatus::AwaitingApproval {
            repo::finish_task(&state.db, &task_id, TaskStatus::Cancelled, Some("rejected by user"))?;
            repo::audit(&state.db, Some(&task_id), "rejected", "")?;
            if let Some(conv) = &task.conversation_id {
                let note = "No problem, I set that plan aside. Nothing was changed.";
                conversations::add_message(&state.db, conv, Role::Assistant, note, None)?;
            }
        }
    }
    repo::get_task(&state.db, &task_id)
}

#[tauri::command]
pub fn undo_task(state: State<'_, AppState>, task_id: String) -> Result<TaskView> {
    if state.is_running(&task_id) {
        return Err(AppError::Invalid("wait for the task to finish before undoing it".into()));
    }
    let outcome = executor::undo(&state.db, &task_id)?;
    let task = repo::get_task(&state.db, &task_id)?;
    let restored = task
        .steps
        .iter()
        .filter(|s| s.status == StepStatus::Undone && matches!(s.op, Operation::MoveFile { .. }))
        .count();
    let mut note = format!("Undone. {restored} files are back where they were.");
    if outcome.failed > 0 {
        note.push_str(&format!(
            " {} item(s) couldn’t be restored, usually a folder that now holds other files. I left those in place.",
            outcome.failed
        ));
    }
    if let Some(conv) = &task.conversation_id {
        conversations::add_message(&state.db, conv, Role::Assistant, &note, None)?;
    }
    Ok(task)
}

#[tauri::command]
pub fn get_task(state: State<'_, AppState>, task_id: String) -> Result<TaskView> {
    repo::get_task(&state.db, &task_id)
}

#[cfg(test)]
mod live {
    use super::*;

    /// The whole chat flow against the real local model: `pnpm test:ollama`.
    #[test]
    #[ignore]
    fn live_chat_flow_end_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(dir.path()).unwrap();
        for n in ["Invoice_Oct.pdf", "IMG_0001.JPG", "IMG_0002.png", "Zoom.pkg", "notes.txt", "budget.xlsx"] {
            std::fs::write(root.join(n), "x").unwrap();
        }
        let db = Db::open_in_memory().unwrap();
        let cancel = AtomicBool::new(false);
        let log = |stage: u8, label: String| println!("  [stage {stage}] {label}");
        let ask = |conv_id: &str, text: &str| {
            conversations::add_message(&db, conv_id, Role::User, text, None).unwrap();
            let conv = conversations::get(&db, conv_id).unwrap();
            let persona = personas::get(&conv.persona);
            let t = std::time::Instant::now();
            let first = std::sync::Mutex::new(None::<std::time::Duration>);
            let on_reply = |_: &str| {
                first.lock().unwrap().get_or_insert(t.elapsed());
            };
            let r = tauri::async_runtime::block_on(respond(&db, &conv, text, &Ollama::for_persona(persona.id), persona, &cancel, &log, &on_reply)).unwrap();
            if let Some(f) = *first.lock().unwrap() {
                println!("  first words after {f:.1?}");
            }
            println!("> [{}] {text}\n< {} ({:.1?})\n", persona.name, r.0, t.elapsed());
            r
        };

        // Each model introduces itself in its own voice, and gets the product facts right.
        for p in personas::PERSONAS {
            let conv = conversations::create(&db, "default", p.id).unwrap();
            assert!(ask(&conv.id, "Hi! Who are you and what can you do?").1.is_none());
            ask(&conv.id, "can you run without internet");
            ask(&conv.id, "do you know me?");
        }

        let conv = conversations::create(&db, "default", "ario").unwrap();
        assert!(ask(&conv.id, "Summarize my lecture PDFs into study notes").1.is_none());
        assert!(ask(&conv.id, "Organize my Downloads folder").1.is_none());

        // With a folder attached, an organize request produces a plan awaiting approval.
        let grant = repo::upsert_grant(&db, &root.display().to_string()).unwrap();
        conversations::set_grant(&db, &conv.id, Some(&grant.id)).unwrap();
        let (_, task_id) = ask(&conv.id, "Sort this folder by file type");
        let task_id = task_id.expect("a plan");
        assert_eq!(repo::get_task(&db, &task_id).unwrap().status, TaskStatus::AwaitingApproval);

        assert_eq!(executor::execute(&db, &task_id, &AtomicBool::new(false)).unwrap(), TaskStatus::Completed);
        executor::undo(&db, &task_id).unwrap();
        let names: Vec<_> = std::fs::read_dir(&root).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names.len(), 6, "everything is back at the top level: {names:?}");
    }
}
