//! The tool registry: the only operations a plan can contain.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::security::permissions::ensure_within;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    CreateFolder { path: PathBuf },
    MoveFile { from: PathBuf, to: PathBuf },
    /// Moves a file into the user's Trash (`to` is its place there), so it
    /// can still be restored, by Undo or from the Trash.
    TrashFile { from: PathBuf, to: PathBuf },
}

impl Operation {
    /// Source and destination of a step that relocates a file.
    pub fn file_move(&self) -> Option<(&Path, &Path)> {
        match self {
            Operation::MoveFile { from, to } | Operation::TrashFile { from, to } => Some((from, to)),
            Operation::CreateFolder { .. } => None,
        }
    }
}

/// The current user's Trash on the startup volume.
pub fn trash_dir() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME")?);
    home.is_absolute().then(|| home.join(".Trash"))
}

/// Why a cleanup plan picked a file, shown on the approval card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CleanReason {
    /// The group on the card: "Duplicates", "Installers", ...
    pub group: String,
    /// One line for this file: "Same as report.pdf".
    pub detail: String,
    pub size: u64,
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
    /// "clean" for a cleanup plan; empty for organizing and renaming.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
    /// Cleanup plans: why each file (by name) is going to the Trash.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reasons: BTreeMap<String, CleanReason>,
    /// Cleanup plans: the space the Trash would hold, in bytes.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub freed_bytes: u64,
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

/// Re-checks an operation against the folder being organized. Plans only ever
/// create folders directly under it, and move a file from it into one of those
/// folders or rename it in place (a move to a new name, never over an existing
/// file). Anything else is rejected.
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
        Operation::TrashFile { from, to } => {
            ensure_within(root, from)?;
            let trash = trash_dir().ok_or_else(|| AppError::Permission("the Trash couldn't be found".into()))?;
            ensure_within(&trash, to)?;
            if !direct_child(from) || to.parent() != Some(trash.as_path()) {
                return Err(AppError::InvalidPlan(format!(
                    "unsupported move to the Trash {} -> {}",
                    from.display(),
                    to.display()
                )));
            }
        }
        Operation::MoveFile { from, to } => {
            ensure_within(root, from)?;
            ensure_within(root, to)?;
            let into_subfolder = to.parent().is_some_and(direct_child);
            let renamed_in_place = direct_child(to) && to != from;
            if !direct_child(from) || !(into_subfolder || renamed_in_place) {
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
            Operation::MoveFile { from: root.join("a.png"), to: root.join("logo-512x512.png") },
            Operation::CreateFolder { path: root.join("Images") },
            Operation::MoveFile { from: root.join("a.png"), to: root.join("Images/a.png") },
        ];
        for op in &ok {
            validate_operation(&root, op).unwrap();
        }
        let bad = [
            Operation::MoveFile { from: root.join("a.png"), to: root.join("a.png") },
            Operation::CreateFolder { path: root.join("a/b") },
            Operation::CreateFolder { path: "/etc/x".into() },
            Operation::MoveFile { from: root.join("Images/a.png"), to: root.join("X/a.png") },
            Operation::MoveFile { from: "/etc/passwd".into(), to: root.join("X/passwd") },
            Operation::MoveFile { from: root.join("a.png"), to: root.join("X/../../a.png") },
        ];
        for op in &bad {
            assert!(validate_operation(&root, op).is_err(), "{op:?}");
        }
        let trash = trash_dir().unwrap();
        validate_operation(&root, &Operation::TrashFile { from: root.join("a.dmg"), to: trash.join("a.dmg") }).unwrap();
        for op in [
            Operation::TrashFile { from: root.join("Images/a.png"), to: trash.join("a.png") },
            Operation::TrashFile { from: root.join("a.dmg"), to: root.join("Trash/a.dmg") },
            Operation::TrashFile { from: root.join("a.dmg"), to: trash.join("sub/a.dmg") },
            Operation::TrashFile { from: "/etc/hosts".into(), to: trash.join("hosts") },
        ] {
            assert!(validate_operation(&root, &op).is_err(), "{op:?}");
        }
    }
}
