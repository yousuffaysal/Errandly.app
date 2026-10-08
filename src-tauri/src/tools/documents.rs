//! Local text extraction for PDF, Word, text and Markdown files (PRD §12).

use std::io::Read;
use std::path::Path;

use crate::error::{AppError, Result};

/// More than enough for a summary; keeps memory and model time bounded.
pub const MAX_CHARS: usize = 200_000;
pub const EXTENSIONS: &[&str] = &["pdf", "docx", "txt", "md", "markdown"];

pub fn is_document(ext: &str) -> bool {
    EXTENSIONS.contains(&ext)
}

/// Extracts readable text. The caller has already checked `path` is inside a
/// granted folder.
pub fn extract_text(path: &Path) -> Result<String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let text = match ext.as_str() {
        "pdf" => pdf(path)?,
        "docx" => docx(path)?,
        "txt" | "md" | "markdown" => String::from_utf8_lossy(&std::fs::read(path)?).into_owned(),
        _ => return Err(AppError::Invalid(format!("{ext} files aren't supported yet"))),
    };
    let text = tidy(&text);
    if text.trim().is_empty() {
        return Err(AppError::Invalid(
            "no readable text found (it may be a scanned PDF; text recognition is coming later)".into(),
        ));
    }
    Ok(text.chars().take(MAX_CHARS).collect())
}

fn pdf(path: &Path) -> Result<String> {
    let path = path.to_path_buf();
    // The PDF parser can panic on malformed files; contain it to this file.
    std::panic::catch_unwind(move || pdf_extract::extract_text(&path))
        .map_err(|_| AppError::Invalid("this PDF couldn't be read".into()))?
        .map_err(|e| AppError::Invalid(format!("this PDF couldn't be read: {e}")))
}

fn docx(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| AppError::Invalid("this Word file couldn't be opened".into()))?;
    let mut xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|_| AppError::Invalid("this Word file has no text".into()))?
        .take(20_000_000)
        .read_to_string(&mut xml)?;
    Ok(xml_text(&xml))
}

/// Paragraph-aware text from WordprocessingML: keeps paragraph and line
/// breaks, drops every tag, and decodes the basic XML entities.
fn xml_text(xml: &str) -> String {
    let xml = xml.replace("</w:p>", "\n").replace("<w:br/>", "\n").replace("<w:tab/>", "\t");
    let mut out = String::with_capacity(xml.len() / 3);
    let mut in_tag = false;
    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// Collapses runs of blank lines and trailing spaces left by extraction.
fn tidy(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank = 0;
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Splits text into chunks of about `size` characters at paragraph breaks.
pub fn chunks(text: &str, size: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for para in text.split("\n\n") {
        if !cur.is_empty() && cur.chars().count() + para.chars().count() > size {
            out.push(std::mem::take(&mut cur));
        }
        if para.chars().count() > size {
            // One very long paragraph: hard-split it.
            let chars: Vec<char> = para.chars().collect();
            for piece in chars.chunks(size) {
                out.push(piece.iter().collect());
            }
            continue;
        }
        if !cur.is_empty() {
            cur.push_str("\n\n");
        }
        cur.push_str(para);
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_text_and_docx_xml() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("notes.md");
        std::fs::write(&p, "# Title\n\n\n\nBody   \n").unwrap();
        assert_eq!(extract_text(&p).unwrap(), "# Title\n\nBody\n");
        assert_eq!(
            xml_text("<w:p><w:r><w:t>Tom &amp; Jerry</w:t></w:r></w:p><w:p><w:t>2 &lt; 3</w:t></w:p>"),
            "Tom & Jerry\n2 < 3\n"
        );
        let empty = d.path().join("empty.txt");
        std::fs::write(&empty, "   \n").unwrap();
        assert!(extract_text(&empty).is_err());
        assert!(extract_text(&d.path().join("x.exe")).is_err());
    }

    #[test]
    fn broken_pdf_is_an_error_not_a_crash() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("broken.pdf");
        std::fs::write(&p, b"%PDF-1.4 garbage").unwrap();
        assert!(extract_text(&p).is_err());
    }

    #[test]
    fn chunks_respect_paragraphs() {
        let text = "a".repeat(40) + "\n\n" + &"b".repeat(40) + "\n\n" + &"c".repeat(120);
        let c = chunks(&text, 100);
        assert_eq!(c.len(), 3);
        assert!(c[0].starts_with('a') && c[0].contains('b'));
        assert!(c.iter().all(|c| c.chars().count() <= 100));
    }
}
