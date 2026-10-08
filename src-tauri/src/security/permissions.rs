//! Path checks enforced outside the model (SEC-001..SEC-004).
//!
//! Every path the executor touches must sit strictly inside a folder the user
//! granted through the native folder picker, with no `..` components and no
//! symbolic links anywhere between the granted root and the target.

use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::error::{AppError, Result};

/// Canonicalizes a folder the user picked. The root itself may not be a symlink.
pub fn canonical_root(path: &Path) -> Result<PathBuf> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Err(AppError::Permission(
            "a granted folder may not be a symbolic link".into(),
        ));
    }
    if !meta.is_dir() {
        return Err(AppError::Permission(format!(
            "{} is not a folder",
            path.display()
        )));
    }
    Ok(std::fs::canonicalize(path)?)
}

/// Ensures `path` is strictly inside the canonical `root`. `path` does not need
/// to exist yet, but any part of it that does exist must not be a symlink.
pub fn ensure_within(root: &Path, path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(AppError::Permission(format!(
            "relative path rejected: {}",
            path.display()
        )));
    }
    if path
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(AppError::Permission(format!(
            "path traversal rejected: {}",
            path.display()
        )));
    }
    if path == root || !path.starts_with(root) {
        return Err(AppError::Permission(format!(
            "{} is outside the granted folder {}",
            path.display(),
            root.display()
        )));
    }

    let mut cur = root.to_path_buf();
    for comp in path.strip_prefix(root).expect("checked above").components() {
        cur.push(comp);
        match std::fs::symlink_metadata(&cur) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(AppError::Permission(format!(
                    "symbolic link rejected: {}",
                    cur.display()
                )))
            }
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => break,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = canonical_root(dir.path()).unwrap();
        (dir, root)
    }

    #[test]
    fn accepts_paths_inside_root() {
        let (_d, root) = root();
        ensure_within(&root, &root.join("a.pdf")).unwrap();
        ensure_within(&root, &root.join("Images").join("a.png")).unwrap();
    }

    #[test]
    fn rejects_root_itself_and_outside_paths() {
        let (_d, root) = root();
        assert!(ensure_within(&root, &root).is_err());
        assert!(ensure_within(&root, Path::new("/etc/passwd")).is_err());
        assert!(ensure_within(&root, Path::new("relative.txt")).is_err());
    }

    #[test]
    fn rejects_traversal() {
        let (_d, root) = root();
        let sneaky = root.join("Images").join("..").join("..").join("x");
        assert!(ensure_within(&root, &sneaky).is_err());
    }

    #[test]
    fn rejects_symlinks_inside_root() {
        let (_d, root) = root();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("link")).unwrap();
        assert!(ensure_within(&root, &root.join("link").join("a.txt")).is_err());
        assert!(canonical_root(&root.join("link")).is_err());
    }
}
