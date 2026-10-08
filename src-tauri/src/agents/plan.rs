//! The tool registry: the only operations a plan can contain.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::security::permissions::ensure_within;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    CreateFolder { path: PathBuf },
    MoveFile { from: PathBuf, to: PathBuf },
}

/// What the planner decided, shown on the approval screen alongside the steps.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlanMeta {
    pub categories: Vec<String>,
    pub scanned_files: usize,
    pub left_in_place: Vec<String>,
    /// Model outputs discarded by validation (unknown file index, duplicate, bad folder).
    pub rejected_outputs: usize,
}

/// Re-checks an operation against the granted root. Phase 0 plans only ever
/// create folders directly under the root and move files from the root into one
/// of those folders, so anything else is rejected.
pub fn validate_operation(root: &Path, op: &Operation) -> Result<()> {
    let direct_child = |p: &Path| p.parent() == Some(root);
    match op {
        Operation::CreateFolder { path } => {
            ensure_within(root, path)?;
            if !direct_child(path) {
                return Err(AppError::InvalidPlan(format!(
                    "folders may only be created directly in {}",
                    root.display()
                )));
            }
        }
        Operation::MoveFile { from, to } => {
            ensure_within(root, from)?;
            ensure_within(root, to)?;
            let into_subfolder = to.parent().is_some_and(direct_child);
            if !direct_child(from) || !into_subfolder {
                return Err(AppError::InvalidPlan(format!(
                    "unsupported move {} -> {}",
                    from.display(),
                    to.display()
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_allows_the_flat_organize_shape() {
        let root = PathBuf::from("/tmp/granted-root-that-need-not-exist");
        let ok = [
            Operation::CreateFolder { path: root.join("Images") },
            Operation::MoveFile { from: root.join("a.png"), to: root.join("Images/a.png") },
        ];
        for op in &ok {
            validate_operation(&root, op).unwrap();
        }
        let bad = [
            Operation::CreateFolder { path: root.join("a/b") },
            Operation::CreateFolder { path: "/etc/x".into() },
            Operation::MoveFile { from: root.join("a.png"), to: root.join("b.png") },
            Operation::MoveFile { from: root.join("Images/a.png"), to: root.join("X/a.png") },
            Operation::MoveFile { from: "/etc/passwd".into(), to: root.join("X/passwd") },
            Operation::MoveFile { from: root.join("a.png"), to: root.join("X/../../a.png") },
        ];
        for op in &bad {
            assert!(validate_operation(&root, op).is_err(), "{op:?}");
        }
    }
}
