//! Tauri commands: the only entry points the UI has into the backend.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::agents::{executor, planner};
use crate::ai::ollama::{self, AiStatus, Ollama};
use crate::error::{AppError, Result};
use crate::security::permissions::canonical_root;
use crate::storage::repo::{self, Grant, TaskStatus, TaskView};
use crate::storage::sqlite::Db;
use crate::tools::files::scan_folder;

const MAX_INSTRUCTION_CHARS: usize = 2000;

pub struct AppState {
    pub db: Arc<Db>,
    /// Cancellation flags for tasks that are planning or executing.
    running: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self { db: Arc::new(db), running: Mutex::default() }
    }

    fn start(&self, task_id: &str) -> Result<Arc<AtomicBool>> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if running.contains_key(task_id) {
            return Err(AppError::Invalid("this task is already running".into()));
        }
        let flag = Arc::new(AtomicBool::new(false));
        running.insert(task_id.to_string(), flag.clone());
        Ok(flag)
    }

    fn finish(&self, task_id: &str) {
        self.running.lock().unwrap_or_else(|e| e.into_inner()).remove(task_id);
    }
}

#[tauri::command]
pub async fn ai_status() -> AiStatus {
    ollama::status().await
}

/// Opens the native folder picker. Only folders chosen here can be granted.
#[tauri::command]
pub async fn pick_and_grant_folder(app: AppHandle, state: State<'_, AppState>) -> Result<Option<Grant>> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("Choose a folder ActionDesk may organize")
            .blocking_pick_folder()
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
    let grant = repo::upsert_grant(&state.db, &root.display().to_string())?;
    repo::audit(&state.db, None, "grant", &grant.path)?;
    Ok(Some(grant))
}

#[tauri::command]
pub fn list_grants(state: State<'_, AppState>) -> Result<Vec<Grant>> {
    repo::list_grants(&state.db)
}

#[tauri::command]
pub fn revoke_grant(state: State<'_, AppState>, grant_id: String) -> Result<()> {
    let grant = repo::get_grant(&state.db, &grant_id)?;
    repo::delete_grant(&state.db, &grant_id)?;
    repo::audit(&state.db, None, "revoke", &grant.path)
}

/// Scans the granted folder and asks the local model for a plan. Nothing on
/// disk changes; the task ends up awaiting approval (or failed).
#[tauri::command]
pub async fn plan_task(
    state: State<'_, AppState>,
    instruction: String,
    grant_id: String,
    model: String,
) -> Result<TaskView> {
    let instruction = instruction.trim().to_string();
    if instruction.is_empty() || instruction.chars().count() > MAX_INSTRUCTION_CHARS {
        return Err(AppError::Invalid(format!(
            "describe the task in 1 to {MAX_INSTRUCTION_CHARS} characters"
        )));
    }
    let grant = repo::get_grant(&state.db, &grant_id)?;
    let root = PathBuf::from(&grant.path);
    if canonical_root(&root)? != root {
        return Err(AppError::Permission("the granted folder has moved; grant it again".into()));
    }
    let files = scan_folder(&root)?;

    let task_id = repo::create_task(&state.db, &instruction, &model, &grant.path)?;
    let cancel = state.start(&task_id)?;
    let planned =
        planner::plan_organize(&Ollama::new(model), &instruction, &root, &files, &cancel).await;
    state.finish(&task_id);

    match planned {
        Ok(p) => {
            let meta = serde_json::to_string(&p.meta).expect("plan meta serializes");
            repo::save_plan(&state.db, &task_id, &meta, &p.operations)?;
        }
        Err(e) => {
            let status = if cancel.load(Ordering::Relaxed) { TaskStatus::Cancelled } else { TaskStatus::Failed };
            repo::finish_task(&state.db, &task_id, status, Some(&e.to_string()))?;
        }
    }
    repo::get_task(&state.db, &task_id)
}

/// The user approved the plan shown to them: run it.
#[tauri::command]
pub async fn approve_task(state: State<'_, AppState>, task_id: String) -> Result<TaskView> {
    let cancel = state.start(&task_id)?;
    let db = state.db.clone();
    let id = task_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || executor::execute(&db, &id, &cancel))
        .await
        .map_err(|e| AppError::Invalid(e.to_string()));
    state.finish(&task_id);
    result??;
    repo::get_task(&state.db, &task_id)
}

/// Rejects a plan awaiting approval, or asks a running task to stop after its current step.
#[tauri::command]
pub fn cancel_task(state: State<'_, AppState>, task_id: String) -> Result<TaskView> {
    let flag = state.running.lock().unwrap_or_else(|e| e.into_inner()).get(&task_id).cloned();
    match flag {
        Some(flag) => flag.store(true, Ordering::Relaxed),
        None => {
            let task = repo::get_task(&state.db, &task_id)?;
            if task.status == TaskStatus::AwaitingApproval {
                repo::finish_task(&state.db, &task_id, TaskStatus::Cancelled, Some("rejected by user"))?;
                repo::audit(&state.db, Some(&task_id), "rejected", "")?;
            }
        }
    }
    repo::get_task(&state.db, &task_id)
}

#[tauri::command]
pub fn undo_task(state: State<'_, AppState>, task_id: String) -> Result<TaskView> {
    if state.running.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&task_id) {
        return Err(AppError::Invalid("wait for the task to finish before undoing it".into()));
    }
    executor::undo(&state.db, &task_id)?;
    repo::get_task(&state.db, &task_id)
}

#[tauri::command]
pub fn get_task(state: State<'_, AppState>, task_id: String) -> Result<TaskView> {
    repo::get_task(&state.db, &task_id)
}

#[tauri::command]
pub fn list_tasks(state: State<'_, AppState>) -> Result<Vec<TaskView>> {
    repo::list_tasks(&state.db, 50)
}
