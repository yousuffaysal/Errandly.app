//! Executes approved plans, reconciles interrupted ones and undoes finished ones.
//!
//! Each step is journaled as `started` before its file-system call and as
//! `done`/`failed` after it, so after a crash we can tell from the journal and
//! the disk exactly which operations happened.

use std::io::ErrorKind;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use super::plan::{validate_operation, Operation};
use super::verifier;
use crate::error::{AppError, Result};
use crate::storage::repo::{self, StepStatus, TaskStatus};
use crate::storage::sqlite::Db;
use crate::tools::files::{move_no_overwrite, unique_destination};

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub done: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Runs every pending step of an approved task, then verifies and sets the
/// final status. Returns that status.
pub fn execute(db: &Db, task_id: &str, cancel: &AtomicBool) -> Result<TaskStatus> {
    let task = repo::get_task(db, task_id)?;
    if task.status != TaskStatus::AwaitingApproval {
        return Err(AppError::Invalid(format!(
            "task is {}, not awaiting approval",
            task.status.as_str()
        )));
    }
    // The grant may have been revoked between planning and approval.
    if !repo::is_granted(db, &task.root)? {
        repo::finish_task(db, task_id, TaskStatus::Failed, Some("folder access was revoked"))?;
        return Err(AppError::Permission("folder access was revoked".into()));
    }
    let root = Path::new(&task.root);
    repo::set_status(db, task_id, TaskStatus::Executing)?;
    repo::audit(db, Some(task_id), "execute_started", &task.root)?;

    let mut outcome = Outcome::default();
    let mut cancelled = false;
    for step in task.steps.iter().filter(|s| s.status == StepStatus::Pending) {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
        }
        if cancelled {
            repo::update_step(db, task_id, step.seq, StepStatus::Skipped, &step.op, Some("cancelled"))?;
            outcome.skipped += 1;
            continue;
        }
        match run_step(db, task_id, step.seq, root, &step.op) {
            Ok(StepStatus::Done) => outcome.done += 1,
            // A folder that already existed: nothing to do, and nothing to undo.
            Ok(_) => {}
            Err(e) => {
                outcome.failed += 1;
                repo::update_step(db, task_id, step.seq, StepStatus::Failed, &step.op, Some(&e.to_string()))?;
            }
        }
    }

    repo::set_status(db, task_id, TaskStatus::Verifying)?;
    let report = verifier::verify(db, task_id)?;
    let status = final_status(&outcome, cancelled, report.mismatches.len());
    let error = (!report.mismatches.is_empty()).then(|| report.mismatches.join("; "));
    repo::finish_task(db, task_id, status, error.as_deref())?;
    repo::audit(
        db,
        Some(task_id),
        "execute_finished",
        &format!("{} done={} failed={} skipped={}", status.as_str(), outcome.done, outcome.failed, outcome.skipped),
    )?;
    Ok(status)
}

fn final_status(o: &Outcome, cancelled: bool, mismatches: usize) -> TaskStatus {
    if o.failed == 0 && o.skipped == 0 && mismatches == 0 && !cancelled {
        TaskStatus::Completed
    } else if o.done == 0 && cancelled {
        TaskStatus::Cancelled
    } else if o.done == 0 {
        TaskStatus::Failed
    } else {
        TaskStatus::PartiallyCompleted
    }
}

/// Performs one journaled step. Returns `Done`, or `Skipped` for a folder
/// that already existed (and therefore must never be removed by undo).
fn run_step(db: &Db, task_id: &str, seq: i64, root: &Path, op: &Operation) -> Result<StepStatus> {
    validate_operation(root, op)?;
    match op {
        Operation::CreateFolder { path } => {
            repo::update_step(db, task_id, seq, StepStatus::Started, op, None)?;
            match std::fs::create_dir(path) {
                Ok(()) => {
                    repo::update_step(db, task_id, seq, StepStatus::Done, op, None)?;
                    Ok(StepStatus::Done)
                }
                Err(e) if e.kind() == ErrorKind::AlreadyExists && path.is_dir() => {
                    repo::update_step(db, task_id, seq, StepStatus::Skipped, op, Some("folder already existed"))?;
                    Ok(StepStatus::Skipped)
                }
                Err(e) => Err(e.into()),
            }
        }
        Operation::MoveFile { from, to } => {
            let meta = std::fs::symlink_metadata(from)?;
            if !meta.file_type().is_file() {
                return Err(AppError::Permission(format!("{} is no longer a regular file", from.display())));
            }
            // Something may have appeared at the destination since planning.
            let mut op = op.clone();
            if std::fs::symlink_metadata(to).is_ok() {
                let dir = to.parent().expect("validated move has a parent");
                // Keep the planned (possibly new) name, just made unique.
                let name = to.file_name().and_then(|n| n.to_str()).expect("planned names are UTF-8");
                op = Operation::MoveFile { from: from.clone(), to: unique_destination(dir, name, &Default::default()) };
                validate_operation(root, &op)?;
            }
            let Operation::MoveFile { to, .. } = &op else { unreachable!() };
            repo::update_step(db, task_id, seq, StepStatus::Started, &op, None)?;
            move_no_overwrite(from, to)?;
            repo::update_step(db, task_id, seq, StepStatus::Done, &op, None)?;
            Ok(StepStatus::Done)
        }
    }
}

/// Run at startup: settles tasks that were executing when the app stopped (T-009).
pub fn reconcile_interrupted(db: &Db) -> Result<usize> {
    let ids = repo::task_ids_with_status(db, &[TaskStatus::Executing, TaskStatus::Verifying])?;
    for id in &ids {
        let mut outcome = Outcome::default();
        for step in repo::steps(db, id)? {
            let (status, note) = match step.status {
                StepStatus::Started => {
                    let happened = match &step.op {
                        Operation::CreateFolder { path } => path.is_dir(),
                        Operation::MoveFile { from, to } => to.exists() && !from.exists(),
                    };
                    if happened { (StepStatus::Done, None) } else { (StepStatus::Failed, Some("interrupted")) }
                }
                StepStatus::Pending => (StepStatus::Skipped, Some("interrupted")),
                other => (other, None),
            };
            if status != step.status {
                repo::update_step(db, id, step.seq, status, &step.op, note)?;
            }
            match status {
                StepStatus::Done => outcome.done += 1,
                StepStatus::Failed => outcome.failed += 1,
                _ => outcome.skipped += 1,
            }
        }
        let status = if outcome.done > 0 { TaskStatus::PartiallyCompleted } else { TaskStatus::Failed };
        repo::finish_task(db, id, status, Some("Errandly stopped while this task was running"))?;
        repo::audit(db, Some(id), "reconciled", status.as_str())?;
    }
    Ok(ids.len())
}

/// Reverses a task's completed steps, newest first (T-015). Moves go back only
/// if the original location is still free; folders are removed only if the task
/// created them and they are empty.
pub fn undo(db: &Db, task_id: &str) -> Result<Outcome> {
    let task = repo::get_task(db, task_id)?;
    if task.undone_at.is_some() {
        return Err(AppError::Invalid("this task was already undone".into()));
    }
    if !matches!(task.status, TaskStatus::Completed | TaskStatus::PartiallyCompleted | TaskStatus::Failed | TaskStatus::Cancelled) {
        return Err(AppError::Invalid(format!("cannot undo a task that is {}", task.status.as_str())));
    }
    let root = Path::new(&task.root);
    let mut outcome = Outcome::default();
    for step in task.steps.iter().rev().filter(|s| s.status == StepStatus::Done) {
        let result = validate_operation(root, &step.op).and_then(|_| match &step.op {
            Operation::MoveFile { from, to } => {
                if std::fs::symlink_metadata(from).is_ok() {
                    return Err(AppError::Invalid(format!("{} is occupied", from.display())));
                }
                Ok(move_no_overwrite(to, from)?)
            }
            Operation::CreateFolder { path } => Ok(std::fs::remove_dir(path)?),
        });
        match result {
            Ok(()) => {
                outcome.done += 1;
                repo::update_step(db, task_id, step.seq, StepStatus::Undone, &step.op, None)?;
            }
            Err(e) => {
                outcome.failed += 1;
                let msg = match &e {
                    AppError::Io(io) if matches!(step.op, Operation::CreateFolder { .. }) && io.kind() == ErrorKind::DirectoryNotEmpty => {
                        "kept: folder is not empty".to_string()
                    }
                    _ => e.to_string(),
                };
                repo::update_step(db, task_id, step.seq, StepStatus::UndoFailed, &step.op, Some(&msg))?;
            }
        }
    }
    repo::mark_undone(db, task_id)?;
    repo::audit(db, Some(task_id), "undo", &format!("restored={} failed={}", outcome.done, outcome.failed))?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::planner::build_operations;
    use crate::tools::files::scan_folder;
    use std::collections::HashMap;
    use std::fs;

    struct Fixture {
        _dir: tempfile::TempDir,
        root: std::path::PathBuf,
        db: Db,
        task: String,
    }

    /// A granted folder with a planned (awaiting approval) task that sorts
    /// a.pdf and b.pdf into Docs and c.png into Images.
    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        for (n, body) in [("a.pdf", "A"), ("b.pdf", "B"), ("c.png", "C"), ("keep.txt", "K")] {
            fs::write(root.join(n), body).unwrap();
        }
        let db = Db::open_in_memory().unwrap();
        let root_s = root.display().to_string();
        repo::upsert_grant(&db, &root_s).unwrap();
        let files = scan_folder(&root).unwrap();
        let assignment: HashMap<usize, String> =
            [(0, "Docs"), (1, "Docs"), (2, "Images")].into_iter().map(|(i, f)| (i, f.to_string())).collect();
        let (ops, _) = build_operations(&root, &files, &assignment).unwrap();
        let task = repo::create_task(&db, "organize", "test", &root_s, None).unwrap();
        repo::save_plan(&db, &task, "{}", &ops).unwrap();
        Fixture { _dir: dir, root, db, task }
    }

    fn listing(root: &Path) -> Vec<String> {
        let mut out = Vec::new();
        fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) {
            for e in fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                out.push(p.strip_prefix(base).unwrap().display().to_string());
                if p.is_dir() {
                    walk(base, &p, out);
                }
            }
        }
        walk(root, root, &mut out);
        out.sort();
        out
    }

    #[test]
    fn executes_verifies_and_undoes() {
        let f = fixture();
        let before = listing(&f.root);
        let status = execute(&f.db, &f.task, &AtomicBool::new(false)).unwrap();
        assert_eq!(status, TaskStatus::Completed);
        assert_eq!(
            listing(&f.root),
            ["Docs", "Docs/a.pdf", "Docs/b.pdf", "Images", "Images/c.png", "keep.txt"]
        );

        let undo = undo(&f.db, &f.task).unwrap();
        assert_eq!(undo, Outcome { done: 5, failed: 0, skipped: 0 });
        assert_eq!(listing(&f.root), before);
        assert_eq!(fs::read_to_string(f.root.join("a.pdf")).unwrap(), "A");
        assert!(super::undo(&f.db, &f.task).is_err(), "undo twice");
    }

    /// T-010: a cancelled (rejected) plan changes nothing.
    #[test]
    fn cancel_before_execution_changes_nothing() {
        let f = fixture();
        let before = listing(&f.root);
        let status = execute(&f.db, &f.task, &AtomicBool::new(true)).unwrap();
        assert_eq!(status, TaskStatus::Cancelled);
        assert_eq!(listing(&f.root), before);
    }

    /// T-011: revoking the grant blocks execution.
    #[test]
    fn revoked_grant_blocks_execution() {
        let f = fixture();
        let before = listing(&f.root);
        let grant = repo::list_grants(&f.db).unwrap().remove(0);
        repo::delete_grant(&f.db, &grant.id).unwrap();
        assert!(execute(&f.db, &f.task, &AtomicBool::new(false)).is_err());
        assert_eq!(listing(&f.root), before);
    }

    /// T-019: a file that appears at the destination after planning is never overwritten.
    #[test]
    fn late_collision_is_renamed_not_overwritten() {
        let f = fixture();
        fs::create_dir(f.root.join("Docs")).unwrap();
        fs::write(f.root.join("Docs/a.pdf"), "someone else's").unwrap();
        let status = execute(&f.db, &f.task, &AtomicBool::new(false)).unwrap();
        // The planned CreateFolder step is skipped because Docs now exists.
        assert_eq!(status, TaskStatus::Completed);
        assert_eq!(fs::read_to_string(f.root.join("Docs/a.pdf")).unwrap(), "someone else's");
        assert_eq!(fs::read_to_string(f.root.join("Docs/a (1).pdf")).unwrap(), "A");

        // Undo keeps Docs (not created by this task) and the other file.
        undo(&f.db, &f.task).unwrap();
        assert_eq!(fs::read_to_string(f.root.join("a.pdf")).unwrap(), "A");
        assert_eq!(fs::read_to_string(f.root.join("Docs/a.pdf")).unwrap(), "someone else's");
    }

    #[test]
    fn missing_source_fails_that_step_only() {
        let f = fixture();
        fs::remove_file(f.root.join("b.pdf")).unwrap();
        let status = execute(&f.db, &f.task, &AtomicBool::new(false)).unwrap();
        assert_eq!(status, TaskStatus::PartiallyCompleted);
        let failed: Vec<_> = repo::steps(&f.db, &f.task).unwrap().into_iter().filter(|s| s.status == StepStatus::Failed).collect();
        assert_eq!(failed.len(), 1);
        assert!(f.root.join("Docs/a.pdf").exists() && f.root.join("Images/c.png").exists());
    }

    /// Undo leaves a created folder in place if the user has since added files to it.
    #[test]
    fn undo_keeps_non_empty_created_folders() {
        let f = fixture();
        execute(&f.db, &f.task, &AtomicBool::new(false)).unwrap();
        fs::write(f.root.join("Images/new.png"), "N").unwrap();
        let o = undo(&f.db, &f.task).unwrap();
        assert_eq!(o, Outcome { done: 4, failed: 1, skipped: 0 });
        assert!(f.root.join("c.png").exists() && f.root.join("Images/new.png").exists());
    }

    /// T-009: a crash between journal writes is settled from the journal plus the disk.
    #[test]
    fn reconciles_interrupted_task() {
        let f = fixture();
        repo::set_status(&f.db, &f.task, TaskStatus::Executing).unwrap();
        let steps = repo::steps(&f.db, &f.task).unwrap();
        // Simulate: step 0 (create Docs) done, step 1 (move a.pdf) happened but
        // the 'done' write was lost, step 2 started but the move never ran.
        fs::create_dir(f.root.join("Docs")).unwrap();
        repo::update_step(&f.db, &f.task, 0, StepStatus::Done, &steps[0].op, None).unwrap();
        repo::update_step(&f.db, &f.task, 1, StepStatus::Started, &steps[1].op, None).unwrap();
        fs::rename(f.root.join("a.pdf"), f.root.join("Docs/a.pdf")).unwrap();
        repo::update_step(&f.db, &f.task, 2, StepStatus::Started, &steps[2].op, None).unwrap();

        assert_eq!(reconcile_interrupted(&f.db).unwrap(), 1);
        let t = repo::get_task(&f.db, &f.task).unwrap();
        assert_eq!(t.status, TaskStatus::PartiallyCompleted);
        let st: Vec<_> = t.steps.iter().map(|s| s.status).collect();
        assert_eq!(&st[..3], &[StepStatus::Done, StepStatus::Done, StepStatus::Failed]);
        assert!(st[3..].iter().all(|s| *s == StepStatus::Skipped));

        undo(&f.db, &f.task).unwrap();
        assert!(f.root.join("a.pdf").exists() && !f.root.join("Docs").exists());
    }
}
