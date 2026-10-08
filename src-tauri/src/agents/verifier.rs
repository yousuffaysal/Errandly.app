//! Checks the disk against what the journal says happened (AGENT-010).

use serde::Serialize;

use super::plan::Operation;
use crate::error::Result;
use crate::storage::repo::{self, StepStatus};
use crate::storage::sqlite::Db;

#[derive(Serialize, Debug, Default)]
pub struct Report {
    pub verified: usize,
    pub mismatches: Vec<String>,
}

pub fn verify(db: &Db, task_id: &str) -> Result<Report> {
    let mut report = Report::default();
    for step in repo::steps(db, task_id)?.iter().filter(|s| s.status == StepStatus::Done) {
        let ok = match &step.op {
            Operation::CreateFolder { path } => path.is_dir(),
            Operation::MoveFile { from, to } => to.is_file() && std::fs::symlink_metadata(from).is_err(),
        };
        if ok {
            report.verified += 1;
        } else {
            report.mismatches.push(match &step.op {
                Operation::CreateFolder { path } => format!("folder missing: {}", path.display()),
                Operation::MoveFile { to, .. } => format!("file not at destination: {}", to.display()),
            });
        }
    }
    Ok(report)
}
