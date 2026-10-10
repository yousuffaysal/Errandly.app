//! "Clean up my Downloads": finds files that are safe to let go of and plans
//! moving them to the Trash. Decided by code, never by the model, and every
//! file comes with the reason it was picked:
//!
//! - exact duplicates (same bytes), keeping the original
//! - installers for apps that are already installed, or long forgotten
//! - archives that have already been unzipped next to themselves
//! - unfinished browser downloads
//!
//! Nothing is deleted: files go to the Trash, and Undo brings them back.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hasher;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::plan::{trash_dir, validate_operation, CleanReason, Operation, PlanMeta};
use super::planner::Planned;
use crate::error::{AppError, Result};
use crate::tools::files::{subfolders, unique_destination, FileEntry};

const DAY: Duration = Duration::from_secs(24 * 60 * 60);
/// An installer whose app can't be found is only suggested after this long.
const OLD_INSTALLER_DAYS: u64 = 30;

pub const DUPLICATES: &str = "Duplicates";
pub const INSTALLERS: &str = "Installers";
pub const UNZIPPED: &str = "Already unzipped";
pub const UNFINISHED: &str = "Unfinished downloads";

/// The request is about freeing space or clearing out clutter, rather than
/// sorting files into folders.
pub fn wants_cleanup(request: &str, root: &Path) -> bool {
    let r = request.to_lowercase().replace('’', "'");
    let sorting = ["organi", "sort", "folder by", "group", "arrange", "renam"].iter().any(|w| r.contains(w));
    let explicit = [
        "duplicate", "free up", "free space", "disk space", "storage", "junk", "trash", "delete", "remove",
        "get rid", "old installer", "leftover", "clear out", "clean out", "clutter",
    ]
    .iter()
    .any(|w| r.contains(w));
    let in_downloads = r.contains("download")
        || root.file_name().is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("downloads"));
    explicit || (!sorting && r.contains("clean") && in_downloads)
}

/// Apps installed on this Mac, as lowercase letters and digits only
/// ("Google Chrome.app" → "googlechrome").
pub fn installed_apps() -> Vec<String> {
    let mut dirs = vec![PathBuf::from("/Applications")];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }
    let mut apps = Vec::new();
    for dir in dirs {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(stem) = name.strip_suffix(".app") {
                apps.push(squash(stem));
            }
        }
    }
    apps
}

fn squash(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// The app an installer installs, if it's already on this Mac:
/// "Docker.dmg" and "Docker-arm64-4.31.dmg" → Docker.app.
fn installed_app_for(installer: &str, apps: &[String]) -> Option<String> {
    let stem = installer.rsplit_once('.').map(|(s, _)| s).unwrap_or(installer);
    // The leading words, before versions, architectures and platforms.
    let words: Vec<&str> = stem
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .take_while(|w| {
            let l = w.to_lowercase();
            !w.chars().next().is_some_and(|c| c.is_ascii_digit())
                && !["arm64", "x64", "x86", "amd64", "universal", "darwin", "mac", "macos", "osx", "intel", "setup", "installer", "latest", "release", "stable"]
                    .contains(&l.as_str())
                && !(l.starts_with('v') && l[1..].chars().all(|c| c.is_ascii_digit()) && l.len() > 1)
        })
        .collect();
    let name = squash(&words.concat());
    if name.chars().count() < 3 {
        return None;
    }
    apps.iter()
        .find(|app| app.len() >= 3 && (app.starts_with(&name) || (name.starts_with(app.as_str()) && app.len() >= 4)))
        .map(|_| words.join(" "))
}

/// Copy markers that show which of two identical files is the copy.
fn copy_score(name: &str) -> usize {
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name).to_lowercase();
    let mut score = 0;
    if stem.ends_with(')') && stem.rsplit_once(" (").is_some_and(|(_, n)| n.trim_end_matches(')').chars().all(|c| c.is_ascii_digit())) {
        score += 2;
    }
    if stem.contains("copy") {
        score += 2;
    }
    if stem.ends_with("-1") || stem.ends_with("_1") || stem.ends_with("-2") || stem.ends_with("_2") {
        score += 1;
    }
    score
}

fn same_contents(a: &Path, b: &Path) -> bool {
    let (Ok(mut fa), Ok(mut fb)) = (std::fs::File::open(a), std::fs::File::open(b)) else { return false };
    let (mut ba, mut bb) = (vec![0u8; 64 * 1024], vec![0u8; 64 * 1024]);
    loop {
        let (Ok(na), Ok(nb)) = (read_full(&mut fa, &mut ba), read_full(&mut fb, &mut bb)) else { return false };
        if na != nb || ba[..na] != bb[..nb] {
            return false;
        }
        if na == 0 {
            return true;
        }
    }
}

fn read_full(f: &mut std::fs::File, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut n = 0;
    while n < buf.len() {
        match f.read(&mut buf[n..])? {
            0 => break,
            k => n += k,
        }
    }
    Ok(n)
}

/// A quick fingerprint (start, middle and end of the file) to group likely
/// duplicates before comparing them byte for byte.
fn fingerprint(path: &Path, size: u64) -> Option<u64> {
    use std::io::{Seek, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let mut buf = vec![0u8; 16 * 1024];
    for at in [0, size / 2, size.saturating_sub(buf.len() as u64)] {
        f.seek(SeekFrom::Start(at)).ok()?;
        let n = read_full(&mut f, &mut buf).ok()?;
        h.write(&buf[..n]);
    }
    Some(h.finish())
}

/// Groups of identical files (two or more each), largest first.
fn duplicate_groups(root: &Path, files: &[FileEntry]) -> Vec<Vec<usize>> {
    let mut by_size: HashMap<u64, Vec<usize>> = HashMap::new();
    for (i, f) in files.iter().enumerate() {
        if f.size > 0 {
            by_size.entry(f.size).or_default().push(i);
        }
    }
    let mut groups = Vec::new();
    for (size, same_size) in by_size.into_iter().filter(|(_, v)| v.len() > 1) {
        let mut by_print: HashMap<u64, Vec<usize>> = HashMap::new();
        for i in same_size {
            if let Some(p) = fingerprint(&root.join(&files[i].name), size) {
                by_print.entry(p).or_default().push(i);
            }
        }
        for candidates in by_print.into_values().filter(|v| v.len() > 1) {
            // Confirm byte for byte; a candidate that differs starts its own group.
            let mut remaining = candidates;
            while let Some(first) = remaining.first().copied() {
                let (same, rest): (Vec<usize>, Vec<usize>) = remaining
                    .into_iter()
                    .partition(|&j| j == first || same_contents(&root.join(&files[first].name), &root.join(&files[j].name)));
                if same.len() > 1 {
                    groups.push(same);
                }
                remaining = rest;
            }
        }
    }
    groups.sort_by_key(|g| std::cmp::Reverse(files[g[0]].size));
    groups
}

fn age_days(path: &Path, now: SystemTime) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| now.duration_since(t).ok())
        .map_or(0, |d| d.as_secs() / DAY.as_secs())
}

/// Plans moving clutter in `root` to the Trash. `apps` is the list from
/// [`installed_apps`]; `now` is passed in so tests can control file ages.
pub fn plan_cleanup(root: &Path, files: &[FileEntry], apps: &[String], now: SystemTime) -> Result<Planned> {
    let trash = trash_dir().ok_or_else(|| AppError::Invalid("your Trash couldn't be found".into()))?;
    // Moving to the Trash is a rename, so it only works on the same drive.
    let dev = |p: &Path| std::fs::metadata(p).map(|m| m.dev());
    if dev(root)? != dev(&trash)? {
        return Err(AppError::Invalid(
            "this folder is on another drive, and I can only move files to the Trash on your Mac’s own drive. \
             Ask me to organize it instead."
                .into(),
        ));
    }

    let mut picked: BTreeMap<usize, CleanReason> = BTreeMap::new();
    let mut pick = |i: usize, group: &str, detail: String| {
        picked.entry(i).or_insert(CleanReason { group: group.into(), detail, size: files[i].size });
    };

    for group in duplicate_groups(root, files) {
        // Keep the one that looks like the original: no "(1)" or "copy",
        // then the oldest.
        let keep = *group
            .iter()
            .min_by_key(|&&i| (copy_score(&files[i].name), std::cmp::Reverse(age_days(&root.join(&files[i].name), now)), files[i].name.len()))
            .expect("groups have two or more files");
        for &i in group.iter().filter(|&&i| i != keep) {
            pick(i, DUPLICATES, format!("Same as “{}”, which stays", files[keep].name));
        }
    }

    let folders: HashSet<String> = subfolders(root)?.into_iter().map(|f| f.to_lowercase()).collect();
    for (i, f) in files.iter().enumerate() {
        let path = root.join(&f.name);
        let days = age_days(&path, now);
        match f.extension.as_str() {
            "dmg" | "pkg" | "mpkg" => {
                if let Some(app) = installed_app_for(&f.name, apps) {
                    pick(i, INSTALLERS, format!("{app} is already installed"));
                } else if days >= OLD_INSTALLER_DAYS {
                    pick(i, INSTALLERS, format!("Installer downloaded {days} days ago"));
                }
            }
            "zip" | "rar" | "7z" | "tar" | "tgz" | "gz" => {
                let stem = f.name.trim_end_matches(".gz").trim_end_matches(".tar");
                let stem = stem.rsplit_once('.').map(|(s, _)| s).unwrap_or(stem);
                if folders.contains(&stem.to_lowercase()) {
                    pick(i, UNZIPPED, format!("Already unzipped into “{stem}”"));
                }
            }
            "crdownload" | "download" | "part" | "partial" | "opdownload" if days >= 1 => {
                pick(i, UNFINISHED, "A download that never finished".into());
            }
            _ => {}
        }
    }

    let mut operations = Vec::new();
    let mut claimed = HashSet::new();
    let mut reasons = BTreeMap::new();
    for (i, reason) in picked {
        let to = unique_destination(&trash, &files[i].name, &claimed);
        claimed.insert(to.clone());
        let op = Operation::TrashFile { from: root.join(&files[i].name), to };
        validate_operation(root, &op)?;
        operations.push(op);
        reasons.insert(files[i].name.clone(), reason);
    }
    let freed_bytes = reasons.values().map(|r| r.size).sum();
    let mut categories: Vec<String> = reasons.values().map(|r| r.group.clone()).collect();
    categories.sort();
    categories.dedup();
    Ok(Planned {
        meta: PlanMeta {
            categories,
            scanned_files: files.len(),
            kind: "clean".into(),
            reasons,
            freed_bytes,
            ..Default::default()
        },
        operations,
    })
}

pub fn human_size(bytes: u64) -> String {
    match bytes {
        b if b >= 1 << 30 => format!("{:.1} GB", b as f64 / (1u64 << 30) as f64),
        b if b >= 1 << 20 => format!("{:.0} MB", b as f64 / (1u64 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.0} KB", b as f64 / 1024.0),
        b => format!("{b} bytes"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::files::scan_folder;
    use std::fs;

    fn names(p: &Planned) -> Vec<String> {
        p.operations
            .iter()
            .map(|o| match o {
                Operation::TrashFile { from, .. } => from.file_name().unwrap().to_string_lossy().into_owned(),
                other => panic!("cleanup only trashes files: {other:?}"),
            })
            .collect()
    }

    #[test]
    fn recognizes_cleanup_requests() {
        let dl = Path::new("/Users/me/Downloads");
        let elsewhere = Path::new("/Users/me/Work");
        assert!(wants_cleanup("clean up my downloads", elsewhere));
        assert!(wants_cleanup("clean this up", dl));
        assert!(wants_cleanup("find duplicates", elsewhere));
        assert!(wants_cleanup("help me free up space", elsewhere));
        assert!(!wants_cleanup("clean up my desktop", elsewhere), "tidying a folder is organizing");
        assert!(!wants_cleanup("organize my downloads by type", dl));
    }

    #[test]
    fn matches_installers_to_installed_apps() {
        let apps: Vec<String> = ["Docker", "Google Chrome", "zoom.us", "Cursor"].iter().map(|a| squash(a)).collect();
        assert_eq!(installed_app_for("Docker.dmg", &apps).as_deref(), Some("Docker"));
        assert_eq!(installed_app_for("Docker-arm64-4.31.dmg", &apps).as_deref(), Some("Docker"));
        assert_eq!(installed_app_for("googlechrome.dmg", &apps).as_deref(), Some("googlechrome"));
        assert_eq!(installed_app_for("Zoom.pkg", &apps).as_deref(), Some("Zoom"));
        assert_eq!(installed_app_for("Cursor-darwin-universal.dmg", &apps).as_deref(), Some("Cursor"));
        assert_eq!(installed_app_for("Figma.dmg", &apps), None);
        assert_eq!(installed_app_for("v2.dmg", &apps), None);
    }

    #[test]
    fn plans_only_clutter_with_reasons() {
        let d = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(d.path()).unwrap();
        fs::write(root.join("report.pdf"), "the same report").unwrap();
        fs::write(root.join("report (1).pdf"), "the same report").unwrap();
        fs::write(root.join("report copy.pdf"), "the same report").unwrap();
        fs::write(root.join("other.pdf"), "the same rePort").unwrap(); // same size, different bytes
        fs::write(root.join("Docker.dmg"), "installer").unwrap();
        fs::write(root.join("Figma.dmg"), "new installer").unwrap(); // not installed, recent
        fs::write(root.join("photos.zip"), "zip").unwrap();
        fs::create_dir(root.join("photos")).unwrap();
        fs::write(root.join("notes.zip"), "zip, never unzipped").unwrap();
        fs::write(root.join("movie.mp4.crdownload"), "half").unwrap();
        fs::write(root.join("keep.txt"), "mine").unwrap();
        let files = scan_folder(&root).unwrap();

        let apps = vec![squash("Docker")];
        // Two days on, the unfinished download counts; Figma is still recent.
        let now = SystemTime::now() + 2 * DAY;
        let p = plan_cleanup(&root, &files, &apps, now).unwrap();
        let mut got = names(&p);
        got.sort();
        assert_eq!(got, ["Docker.dmg", "movie.mp4.crdownload", "photos.zip", "report (1).pdf", "report copy.pdf"]);
        assert_eq!(p.meta.kind, "clean");
        assert_eq!(p.meta.reasons["report (1).pdf"].detail, "Same as “report.pdf”, which stays");
        assert_eq!(p.meta.reasons["Docker.dmg"].detail, "Docker is already installed");
        assert_eq!(p.meta.reasons["photos.zip"].group, UNZIPPED);
        assert_eq!(p.meta.freed_bytes, p.meta.reasons.values().map(|r| r.size).sum::<u64>());
        for op in &p.operations {
            validate_operation(&root, op).unwrap();
        }

        // Months later, the forgotten installer is suggested too.
        let later = plan_cleanup(&root, &files, &apps, SystemTime::now() + 40 * DAY).unwrap();
        assert!(names(&later).contains(&"Figma.dmg".to_string()));
    }

    /// The whole flow on disk: to the Trash, then back with Undo.
    #[test]
    fn trashes_and_undoes() {
        use crate::agents::executor::{execute, undo};
        use crate::storage::{repo, sqlite::Db};
        use std::sync::atomic::AtomicBool;

        let d = tempfile::tempdir_in(std::env::var("HOME").unwrap()).unwrap();
        let root = fs::canonicalize(d.path()).unwrap();
        let unique = uuid::Uuid::new_v4().to_string();
        let a = format!("errandly-test-{unique}.txt");
        let b = format!("errandly-test-{unique} (1).txt");
        fs::write(root.join(&a), "same").unwrap();
        fs::write(root.join(&b), "same").unwrap();
        let files = scan_folder(&root).unwrap();
        let p = plan_cleanup(&root, &files, &[], SystemTime::now()).unwrap();
        assert_eq!(names(&p), std::slice::from_ref(&b));

        let db = Db::open_in_memory().unwrap();
        let root_s = root.display().to_string();
        repo::upsert_grant(&db, &root_s).unwrap();
        let task = repo::create_task(&db, "clean", "test", &root_s, None).unwrap();
        repo::save_plan(&db, &task, "{}", &p.operations).unwrap();
        let status = execute(&db, &task, &AtomicBool::new(false)).unwrap();
        assert_eq!(status, repo::TaskStatus::Completed);
        assert!(!root.join(&b).exists() && root.join(&a).exists());
        let in_trash = trash_dir().unwrap().join(&b);
        assert!(in_trash.exists(), "the copy is in the Trash");

        undo(&db, &task).unwrap();
        assert!(root.join(&b).exists() && !in_trash.exists(), "Undo brings it back");
    }
}
