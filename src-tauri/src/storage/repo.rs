//! Typed queries over the local database.

use std::path::PathBuf;

use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;

use super::sqlite::Db;
use crate::agents::plan::Operation;
use crate::error::{AppError, Result};

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $s:literal),+ $(,)? }) => {
        #[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name { $(#[serde(rename = $s)] $variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }
            pub fn parse(s: &str) -> rusqlite::Result<Self> {
                match s {
                    $($s => Ok(Self::$variant),)+
                    other => Err(rusqlite::Error::InvalidColumnType(
                        0, format!("unknown {} {other:?}", stringify!($name)), rusqlite::types::Type::Text)),
                }
            }
        }
    };
}

// Mirrors the PRD's TaskStatus (§10.4).
string_enum!(TaskStatus {
    Created => "created",
    Planning => "planning",
    AwaitingApproval => "awaiting_approval",
    Executing => "executing",
    Verifying => "verifying",
    Completed => "completed",
    PartiallyCompleted => "partially_completed",
    Failed => "failed",
    Cancelled => "cancelled",
});

string_enum!(StepStatus {
    Pending => "pending",
    Started => "started",
    Done => "done",
    Failed => "failed",
    Skipped => "skipped",
    Undone => "undone",
    UndoFailed => "undo_failed",
});

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Grant {
    pub id: String,
    pub path: String,
    pub created_at: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub seq: i64,
    #[serde(flatten)]
    pub op: Operation,
    pub status: StepStatus,
    pub error: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskView {
    pub id: String,
    pub instruction: String,
    pub status: TaskStatus,
    pub model_id: String,
    pub root: String,
    pub conversation_id: Option<String>,
    pub plan: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub undone_at: Option<String>,
    pub steps: Vec<Step>,
}

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

// ---- permission grants ----------------------------------------------------

pub fn upsert_grant(db: &Db, path: &str) -> Result<Grant> {
    db.with(|c| {
        c.execute(
            "INSERT OR IGNORE INTO permission_grants (id, path) VALUES (?1, ?2)",
            params![uuid::Uuid::new_v4().to_string(), path],
        )?;
        c.query_row(
            "SELECT id, path, created_at FROM permission_grants WHERE path = ?1",
            [path],
            grant_from_row,
        )
    })
}

pub fn list_grants(db: &Db) -> Result<Vec<Grant>> {
    db.with(|c| {
        c.prepare("SELECT id, path, created_at FROM permission_grants ORDER BY path")?
            .query_map([], grant_from_row)?
            .collect()
    })
}

/// True if `root` is still an active grant. Checked again right before execution.
/// True if `root` is a granted folder or inside one (a subfolder the user's
/// grant already covers).
pub fn is_granted(db: &Db, root: &str) -> Result<bool> {
    let grants: Vec<String> =
        db.with(|c| c.prepare("SELECT path FROM permission_grants")?.query_map([], |r| r.get(0))?.collect())?;
    let root = std::path::Path::new(root);
    Ok(grants.iter().any(|g| root.starts_with(g)))
}

pub fn delete_grant(db: &Db, id: &str) -> Result<()> {
    db.with(|c| c.execute("DELETE FROM permission_grants WHERE id = ?1", [id]))?;
    Ok(())
}

fn grant_from_row(r: &Row) -> rusqlite::Result<Grant> {
    Ok(Grant {
        id: r.get(0)?,
        path: r.get(1)?,
        created_at: r.get(2)?,
    })
}

// ---- tasks ----------------------------------------------------------------

pub fn create_task(
    db: &Db,
    instruction: &str,
    model_id: &str,
    root: &str,
    conversation_id: Option<&str>,
) -> Result<String> {
    let id = uuid::Uuid::new_v4().to_string();
    db.with(|c| {
        c.execute(
            "INSERT INTO tasks (id, workspace_id, instruction, status, model_id, root, conversation_id)
             VALUES (?1, 'default', ?2, ?3, ?4, ?5, ?6)",
            params![id, instruction, TaskStatus::Planning.as_str(), model_id, root, conversation_id],
        )
    })?;
    Ok(id)
}

pub fn set_status(db: &Db, id: &str, status: TaskStatus) -> Result<()> {
    db.with(|c| {
        c.execute(
            "UPDATE tasks SET status = ?2 WHERE id = ?1",
            params![id, status.as_str()],
        )
    })?;
    Ok(())
}

/// Moves a task to a terminal state and stamps `completed_at`.
pub fn finish_task(db: &Db, id: &str, status: TaskStatus, error: Option<&str>) -> Result<()> {
    db.with(|c| {
        c.execute(
            &format!("UPDATE tasks SET status = ?2, error = ?3, completed_at = {NOW} WHERE id = ?1"),
            params![id, status.as_str(), error],
        )
    })?;
    Ok(())
}

pub fn mark_undone(db: &Db, id: &str) -> Result<()> {
    db.with(|c| c.execute(&format!("UPDATE tasks SET undone_at = {NOW} WHERE id = ?1"), [id]))?;
    Ok(())
}

/// Stores the approved-to-be plan and its operations as pending steps, atomically.
pub fn save_plan(db: &Db, id: &str, plan_json: &str, ops: &[Operation]) -> Result<()> {
    db.with(|c| {
        let tx = c.transaction()?;
        tx.execute(
            "UPDATE tasks SET plan_json = ?2, status = ?3 WHERE id = ?1",
            params![id, plan_json, TaskStatus::AwaitingApproval.as_str()],
        )?;
        {
            let mut ins = tx.prepare(
                "INSERT INTO task_steps (task_id, seq, op, src, dst, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for (seq, op) in ops.iter().enumerate() {
                let (kind, src, dst) = op_columns(op);
                ins.execute(params![id, seq as i64, kind, src, dst, StepStatus::Pending.as_str()])?;
            }
        }
        tx.commit()
    })
}

pub fn update_step(
    db: &Db,
    task_id: &str,
    seq: i64,
    status: StepStatus,
    op: &Operation,
    error: Option<&str>,
) -> Result<()> {
    let (_, src, dst) = op_columns(op);
    db.with(|c| {
        c.execute(
            &format!(
                "UPDATE task_steps SET status = ?3, src = ?4, dst = ?5, error = ?6, updated_at = {NOW}
                 WHERE task_id = ?1 AND seq = ?2"
            ),
            params![task_id, seq, status.as_str(), src, dst, error],
        )
    })?;
    Ok(())
}

pub fn steps(db: &Db, task_id: &str) -> Result<Vec<Step>> {
    db.with(|c| {
        c.prepare("SELECT seq, op, src, dst, status, error FROM task_steps WHERE task_id = ?1 ORDER BY seq")?
            .query_map([task_id], |r| {
                let kind: String = r.get(1)?;
                let src: Option<String> = r.get(2)?;
                let dst: PathBuf = PathBuf::from(r.get::<_, String>(3)?);
                let op = match (kind.as_str(), src) {
                    ("create_folder", _) => Operation::CreateFolder { path: dst },
                    ("move_file", Some(src)) => Operation::MoveFile { from: src.into(), to: dst },
                    ("trash_file", Some(src)) => Operation::TrashFile { from: src.into(), to: dst },
                    _ => {
                        return Err(rusqlite::Error::InvalidColumnType(
                            1,
                            format!("bad step {kind}"),
                            rusqlite::types::Type::Text,
                        ))
                    }
                };
                Ok(Step {
                    seq: r.get(0)?,
                    op,
                    status: StepStatus::parse(&r.get::<_, String>(4)?)?,
                    error: r.get(5)?,
                })
            })?
            .collect()
    })
}

pub fn get_task(db: &Db, id: &str) -> Result<TaskView> {
    let mut task = db
        .with(|c| {
            c.query_row(&format!("{TASK_SELECT} WHERE id = ?1"), [id], task_from_row)
                .optional()
        })?
        .ok_or_else(|| AppError::NotFound(format!("task {id}")))?;
    task.steps = steps(db, id)?;
    Ok(task)
}

pub fn task_ids_with_status(db: &Db, statuses: &[TaskStatus]) -> Result<Vec<String>> {
    let wanted: Vec<&str> = statuses.iter().map(|s| s.as_str()).collect();
    db.with(|c| {
        let ids = c
            .prepare("SELECT id, status FROM tasks")?
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(ids
            .into_iter()
            .filter(|(_, s)| wanted.contains(&s.as_str()))
            .map(|(id, _)| id)
            .collect())
    })
}

pub fn audit(db: &Db, task_id: Option<&str>, kind: &str, detail: &str) -> Result<()> {
    db.with(|c| {
        c.execute(
            "INSERT INTO audit_events (task_id, kind, detail) VALUES (?1, ?2, ?3)",
            params![task_id, kind, detail],
        )
    })?;
    Ok(())
}

const TASK_SELECT: &str = "SELECT id, instruction, status, model_id, root, plan_json, error, created_at, completed_at, undone_at, conversation_id FROM tasks";

fn task_from_row(r: &Row) -> rusqlite::Result<TaskView> {
    let plan: Option<String> = r.get(5)?;
    Ok(TaskView {
        id: r.get(0)?,
        instruction: r.get(1)?,
        status: TaskStatus::parse(&r.get::<_, String>(2)?)?,
        model_id: r.get(3)?,
        root: r.get(4)?,
        plan: plan.and_then(|p| serde_json::from_str(&p).ok()),
        error: r.get(6)?,
        created_at: r.get(7)?,
        completed_at: r.get(8)?,
        undone_at: r.get(9)?,
        conversation_id: r.get(10)?,
        steps: Vec::new(),
    })
}

fn op_columns(op: &Operation) -> (&'static str, Option<String>, String) {
    match op {
        Operation::CreateFolder { path } => ("create_folder", None, path.display().to_string()),
        Operation::MoveFile { from, to } => (
            "move_file",
            Some(from.display().to_string()),
            to.display().to_string(),
        ),
        Operation::TrashFile { from, to } => (
            "trash_file",
            Some(from.display().to_string()),
            to.display().to_string(),
        ),
    }
}
