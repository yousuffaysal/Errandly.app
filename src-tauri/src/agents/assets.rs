//! Renaming and brand-asset organizing, done by code so it is instant and
//! exact. The model can't see images, but the files can tell us a lot: real
//! pixel sizes, formats, and the standard sizes icons come in.
//!
//! "Rename my logos and organize them" on a folder of favicon-16.png …
//! favicon-1024.png, favicon.svg and avatar-round-1024.png becomes:
//!   Favicons/   errandly-favicon-16x16.png …
//!   App icons/  apple-touch-icon-180x180.png, android-chrome-192x192.png …
//!   Vector/     errandly-favicon.svg …
//!   Avatars/    errandly-avatar-round-1024x1024.png

use std::collections::HashMap;
use std::path::Path;

use super::plan::PlanMeta;
use super::planner::{build_placements, file_kind, kind_folder, persona_folder, Placement, Planned};
use crate::error::Result;
use crate::tools::files::FileEntry;

/// The request asks for files to be renamed.
pub fn wants_rename(request: &str) -> bool {
    let r = request.to_lowercase();
    ["rename", "renaming", "proper name", "appropriate name", "better name", "good name", "naming", "consistent name"]
        .iter()
        .any(|w| r.contains(w))
}

const ASSET_WORDS: &[&str] = &["logo", "icon", "favicon", "brand", "avatar", "wordmark", "symbol", "mark"];

/// The request or the files are about logos, icons or brand assets.
pub fn is_brand_request(request: &str, root: &Path, files: &[FileEntry]) -> bool {
    let r = request.to_lowercase();
    if ["logo", "icon", "favicon", "brand"].iter().any(|w| r.contains(w)) {
        return true;
    }
    let folder = root.to_string_lossy().to_lowercase();
    let images = files.iter().filter(|f| is_image(&f.extension)).count();
    let asset_named = files.iter().filter(|f| ASSET_WORDS.iter().any(|w| f.name.to_lowercase().contains(w))).count();
    images * 10 >= files.len() * 7 && (asset_named * 2 >= files.len() || ["logo", "icon", "brand"].iter().any(|w| folder.contains(w)))
}

fn is_image(ext: &str) -> bool {
    matches!(file_kind(ext), "image" | "graphic design")
}

/// A brand name from the folder path: the words before "logo(s)", "brand",
/// "icons" or "assets" in this folder's or a parent's name, e.g.
/// ".../Errandly Logos 2/favicon-light-olive" → "errandly".
pub fn brand_name(root: &Path) -> Option<String> {
    for dir in root.ancestors().take(3) {
        let name = dir.file_name()?.to_string_lossy().to_lowercase();
        let words: Vec<&str> = name.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
        if let Some(i) = words.iter().position(|w| ["logo", "logos", "brand", "branding", "icons", "assets"].contains(w)) {
            let brand = words[..i].join("-");
            if !brand.is_empty() && brand.chars().count() <= 30 {
                return Some(brand);
            }
        }
    }
    None
}

/// Pixel size of an image: raster formats from their header, SVG from its
/// width/height or viewBox.
fn dimensions(path: &Path, ext: &str) -> Option<(u32, u32)> {
    if ext == "svg" {
        return svg_dimensions(&std::fs::read_to_string(path).ok()?);
    }
    imagesize::size(path).ok().map(|s| (s.width as u32, s.height as u32))
}

fn svg_dimensions(svg: &str) -> Option<(u32, u32)> {
    let head: String = svg.chars().take(2000).collect();
    let attr = |name: &str| {
        let i = head.find(&format!("{name}=\""))? + name.len() + 2;
        let v: String = head[i..].chars().take_while(|c| *c != '"').collect();
        Some(v)
    };
    let num = |s: &str| s.trim_end_matches("px").trim().parse::<f64>().ok().filter(|n| *n > 0.0).map(|n| n.round() as u32);
    if let (Some(w), Some(h)) = (attr("width").and_then(|w| num(&w)), attr("height").and_then(|h| num(&h))) {
        return Some((w, h));
    }
    // viewBox="min-x min-y width height"; only width and height must be positive.
    let vb: Vec<f64> = attr("viewBox")?.split([' ', ',']).filter(|s| !s.is_empty()).filter_map(|s| s.parse().ok()).collect();
    (vb.len() == 4 && vb[2] > 0.0 && vb[3] > 0.0).then(|| (vb[2].round() as u32, vb[3].round() as u32))
}

/// "Favicon_final (1)-1024@2x" → "favicon": lowercase, hyphens, no copy
/// markers, version noise or trailing size numbers.
pub fn clean_base(stem: &str) -> String {
    let lower = stem.to_lowercase().replace([' ', '_', '.', '@'], "-");
    let lower = lower.replace("(", "-").replace(")", "-");
    let mut words: Vec<&str> = lower.split('-').filter(|w| !w.is_empty()).collect();
    words.retain(|w| !["copy", "final", "new", "edited", "untitled"].contains(w));
    while let Some(last) = words.last() {
        let is_noise = last.chars().all(|c| c.is_ascii_digit())
            || last.ends_with('x') && last[..last.len() - 1].chars().all(|c| c.is_ascii_digit()) && last.len() > 1
            || last.starts_with('@')
            || last.starts_with('v') && last.len() > 1 && last[1..].chars().all(|c| c.is_ascii_digit())
            || is_size_token(last);
        if is_noise && words.len() > 1 {
            words.pop();
        } else {
            break;
        }
    }
    let base = words.join("-");
    if base.is_empty() { "file".into() } else { base }
}

fn is_size_token(w: &str) -> bool {
    let Some((a, b)) = w.split_once('x') else { return false };
    !a.is_empty() && !b.is_empty() && a.chars().all(|c| c.is_ascii_digit()) && b.chars().all(|c| c.is_ascii_digit())
}

fn with_brand(brand: Option<&str>, base: &str) -> String {
    match brand {
        Some(b) if !base.starts_with(b) => format!("{b}-{base}"),
        _ => base.to_string(),
    }
}

/// Folder and name for one brand asset.
fn place_asset(f: &FileEntry, dims: Option<(u32, u32)>, brand: Option<&str>, rename: bool) -> Placement {
    let ext = f.extension.as_str();
    let stem = f.name.rsplit_once('.').map(|(s, _)| s).unwrap_or(&f.name);
    let base = clean_base(stem);
    let lower = base.as_str();
    let size = |(w, h): (u32, u32)| format!("{w}x{h}");
    let named = |name: String| if rename { format!("{name}.{ext}") } else { f.name.clone() };

    let (folder, name) = if !is_image(ext) {
        (kind_folder(file_kind(ext)).map(str::to_owned), named(with_brand(brand, lower)))
    } else if ext == "svg" || ext == "ai" || ext == "eps" || ext == "pdf" {
        ("Vector".to_owned().into(), named(with_brand(brand, lower)))
    } else if ["avatar", "profile", "social"].iter().any(|w| lower.contains(w)) {
        let n = dims.map(|d| format!("{}-{}", with_brand(brand, lower), size(d))).unwrap_or_else(|| with_brand(brand, lower));
        (Some("Avatars".into()), named(n))
    } else {
        match dims {
            Some((w, h)) if w == h && w <= 64 => (Some("Favicons".into()), named(format!("{}-favicon-{}", brand.unwrap_or("site"), size((w, h))))),
            Some((180, 180)) => (Some("App icons".into()), named("apple-touch-icon-180x180".into())),
            Some((w, h)) if w == h && (w == 192 || w == 512) => (Some("App icons".into()), named(format!("android-chrome-{}", size((w, h))))),
            Some((w, h)) if w == h && w >= 128 => {
                let what = if ["favicon", "icon", "logo", "mark", "symbol"].iter().any(|k| lower.contains(k)) { "app-icon" } else { lower };
                (Some("App icons".into()), named(format!("{}-{}", with_brand(brand, what), size((w, h)))))
            }
            Some(d) => (Some("Logos".into()), named(format!("{}-{}", with_brand(brand, lower), size(d)))),
            None => (Some("Logos".into()), named(with_brand(brand, lower))),
        }
    };
    Placement { folder, name }
}

/// Organizes a folder of logos and icons by purpose, renaming consistently when asked.
pub fn plan_brand(root: &Path, files: &[FileEntry], rename: bool) -> Result<Planned> {
    let brand = brand_name(root);
    let mut placements = HashMap::new();
    for (i, f) in files.iter().enumerate() {
        let dims = dimensions(&root.join(&f.name), &f.extension);
        placements.insert(i, place_asset(f, dims, brand.as_deref(), rename));
    }
    finish(root, files, &placements)
}

/// Sorts by file type and gives every file a clean, consistent name.
pub fn plan_clean_names(root: &Path, files: &[FileEntry], persona: &str) -> Result<Planned> {
    let mut placements = HashMap::new();
    for (i, f) in files.iter().enumerate() {
        let stem = f.name.rsplit_once('.').map(|(s, _)| s).unwrap_or(&f.name);
        let name = if f.extension.is_empty() { clean_base(stem) } else { format!("{}.{}", clean_base(stem), f.extension) };
        let folder = persona_folder(persona, &f.name).or_else(|| kind_folder(file_kind(&f.extension))).map(str::to_owned);
        placements.insert(i, Placement { folder, name });
    }
    finish(root, files, &placements)
}

fn finish(root: &Path, files: &[FileEntry], placements: &HashMap<usize, Placement>) -> Result<Planned> {
    let (operations, left_in_place) = build_placements(root, files, placements)?;
    let mut categories: Vec<String> = placements.values().filter_map(|p| p.folder.clone()).collect();
    categories.sort();
    categories.dedup();
    Ok(Planned { meta: PlanMeta { categories, scanned_files: files.len(), left_in_place, ..Default::default() }, operations })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::plan::{validate_operation, Operation};
    use crate::tools::files::scan_folder;

    fn png(w: u32, h: u32) -> Vec<u8> {
        // A minimal PNG header: signature + IHDR with width and height.
        let mut v = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
        v.extend(w.to_be_bytes());
        v.extend(h.to_be_bytes());
        v.extend([8, 6, 0, 0, 0, 0, 0, 0, 0]);
        v
    }

    #[test]
    fn cleans_names() {
        assert_eq!(clean_base("favicon-1024"), "favicon");
        assert_eq!(clean_base("Avatar Round_1024"), "avatar-round");
        assert_eq!(clean_base("Logo final (1) v2"), "logo");
        assert_eq!(clean_base("logo-512x512@2x"), "logo");
        assert_eq!(clean_base("2024"), "2024", "a name that is only a number stays");
        assert_eq!(brand_name(Path::new("/u/Downloads/Errandly Logos 2/favicon-light-olive")).as_deref(), Some("errandly"));
        assert_eq!(brand_name(Path::new("/u/Downloads/random")), None);
        assert_eq!(svg_dimensions(r#"<svg viewBox="0 0 64 64">"#), Some((64, 64)));
        assert_eq!(svg_dimensions(r#"<svg width="120px" height="40px">"#), Some((120, 40)));
    }

    #[test]
    fn brand_kit_is_organized_by_purpose_and_renamed() {
        let dir = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(dir.path()).unwrap().join("Errandly Logos 2");
        std::fs::create_dir(&root).unwrap();
        for s in [16, 32, 48, 180, 192, 512, 1024] {
            std::fs::write(root.join(format!("favicon-{s}.png")), png(s, s)).unwrap();
        }
        std::fs::write(root.join("avatar-round-1024.png"), png(1024, 1024)).unwrap();
        std::fs::write(root.join("favicon.svg"), r#"<svg viewBox="0 0 64 64"></svg>"#).unwrap();
        let files = scan_folder(&root).unwrap();
        assert!(is_brand_request("i want to rename all logo name wit appropriate name and organize", &root, &files));
        assert!(wants_rename("i want to rename all logo name wit appropriate name and organize"));

        let p = plan_brand(&root, &files, true).unwrap();
        let dest: Vec<String> = p
            .operations
            .iter()
            .filter_map(|o| match o {
                Operation::MoveFile { to, .. } => Some(to.strip_prefix(&root).unwrap().display().to_string()),
                _ => None,
            })
            .collect();
        for expected in [
            "Favicons/errandly-favicon-16x16.png",
            "Favicons/errandly-favicon-48x48.png",
            "App icons/apple-touch-icon-180x180.png",
            "App icons/android-chrome-192x192.png",
            "App icons/android-chrome-512x512.png",
            "App icons/errandly-app-icon-1024x1024.png",
            "Avatars/errandly-avatar-round-1024x1024.png",
            "Vector/errandly-favicon.svg",
        ] {
            assert!(dest.contains(&expected.to_string()), "missing {expected} in {dest:?}");
        }
        for op in &p.operations {
            validate_operation(&root, op).unwrap();
        }

        let keep_names = plan_brand(&root, &files, false).unwrap();
        assert!(keep_names.operations.iter().all(|o| match o {
            Operation::MoveFile { from, to } => from.file_name() == to.file_name(),
            _ => true,
        }), "without rename, names stay");
    }

    #[test]
    fn clean_names_never_collide() {
        let dir = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(dir.path()).unwrap();
        for n in ["Report final.pdf", "report (1).pdf", "Notes_v2.txt"] {
            std::fs::write(root.join(n), "x").unwrap();
        }
        let files = scan_folder(&root).unwrap();
        let p = plan_clean_names(&root, &files, "ario").unwrap();
        let dest: Vec<String> = p.operations.iter().filter_map(|o| match o {
            Operation::MoveFile { to, .. } => Some(to.strip_prefix(&root).unwrap().display().to_string()),
            _ => None,
        }).collect();
        assert!(dest.contains(&"Documents/report.pdf".to_string()));
        assert!(dest.contains(&"Documents/report (1).pdf".to_string()), "{dest:?}");
        assert!(dest.contains(&"Documents/notes.txt".to_string()));
    }
}
