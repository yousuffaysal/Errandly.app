//! Local file-system tools. These are the only ways the agent touches files.

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{AppError, Result};

/// Phase 0 handles one flat folder at a time; larger folders are refused
/// rather than silently truncated.
pub const MAX_SCAN_FILES: usize = 1000;

#[derive(Serialize, Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub extension: String,
    pub size: u64,
}

/// Lists regular, non-hidden files directly inside `root`. Folders, symlinks,
/// hidden files and names that are not valid UTF-8 are skipped.
pub fn scan_folder(root: &Path) -> Result<Vec<FileEntry>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let meta = std::fs::symlink_metadata(entry.path())?;
        if !meta.file_type().is_file() {
            continue;
        }
        files.push(FileEntry {
            extension: Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase(),
            name,
            size: meta.len(),
        });
        if files.len() > MAX_SCAN_FILES {
            return Err(AppError::Invalid(format!(
                "this folder has more than {MAX_SCAN_FILES} files; pick a smaller folder for now"
            )));
        }
    }
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(files)
}

/// Returns `dir/name`, or `dir/name (n).ext` if that path exists on disk or is
/// already claimed by an earlier operation in the same plan.
pub fn unique_destination(dir: &Path, name: &str, claimed: &HashSet<PathBuf>) -> PathBuf {
    let candidate = dir.join(name);
    if !taken(&candidate, claimed) {
        return candidate;
    }
    let path = Path::new(name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    let ext = path.extension().and_then(|e| e.to_str());
    (1..)
        .map(|n| match ext {
            Some(ext) => dir.join(format!("{stem} ({n}).{ext}")),
            None => dir.join(format!("{stem} ({n})")),
        })
        .find(|p| !taken(p, claimed))
        .expect("unbounded search always finds a free name")
}

fn taken(path: &Path, claimed: &HashSet<PathBuf>) -> bool {
    // Compare case-insensitively: APFS volumes are case-insensitive by default.
    let lower = path.to_string_lossy().to_lowercase();
    std::fs::symlink_metadata(path).is_ok()
        || claimed
            .iter()
            .any(|c| c.to_string_lossy().to_lowercase() == lower)
}

/// Renames `from` to `to`, failing with `AlreadyExists` instead of replacing an
/// existing file. On macOS the check and the rename are a single atomic call.
pub fn move_no_overwrite(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let c = |p: &Path| {
            CString::new(p.as_os_str().as_bytes())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains NUL"))
        };
        let (from_c, to_c) = (c(from)?, c(to)?);
        // SAFETY: both pointers are valid NUL-terminated strings for the duration of the call.
        let rc = unsafe { libc::renamex_np(from_c.as_ptr(), to_c.as_ptr(), libc::RENAME_EXCL) };
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        if std::fs::symlink_metadata(to).is_ok() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "destination exists"));
        }
        std::fs::rename(from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scan_skips_hidden_dirs_and_symlinks() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("b.PDF"), "x").unwrap();
        fs::write(d.path().join("a.txt"), "hello").unwrap();
        fs::write(d.path().join(".DS_Store"), "").unwrap();
        fs::create_dir(d.path().join("sub")).unwrap();
        std::os::unix::fs::symlink(d.path().join("a.txt"), d.path().join("link.txt")).unwrap();

        let files = scan_folder(d.path()).unwrap();
        let names: Vec<_> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["a.txt", "b.PDF"]);
        assert_eq!(files[0].size, 5);
        assert_eq!(files[1].extension, "pdf");
    }

    #[test]
    fn unique_destination_avoids_disk_and_plan_collisions() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("a.pdf"), "").unwrap();
        let mut claimed = HashSet::new();
        let first = unique_destination(d.path(), "a.pdf", &claimed);
        assert_eq!(first, d.path().join("a (1).pdf"));
        claimed.insert(first);
        assert_eq!(
            unique_destination(d.path(), "A.PDF", &claimed),
            d.path().join("A (2).PDF")
        );
        assert_eq!(
            unique_destination(d.path(), "README", &claimed),
            d.path().join("README")
        );
    }

    #[test]
    fn move_never_overwrites() {
        let d = tempfile::tempdir().unwrap();
        let (a, b) = (d.path().join("a.txt"), d.path().join("b.txt"));
        fs::write(&a, "a").unwrap();
        fs::write(&b, "b").unwrap();
        let err = move_no_overwrite(&a, &b).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(&b).unwrap(), "b");

        let c = d.path().join("c.txt");
        move_no_overwrite(&a, &c).unwrap();
        assert!(!a.exists() && c.exists());
    }
}
