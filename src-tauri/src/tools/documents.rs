//! Local text extraction for PDF, Word, text and Markdown files (PRD §12).

use std::io::Read;
use std::path::Path;

use super::ocr;
use crate::error::{AppError, Result};

/// More than enough for a summary; keeps memory and model time bounded.
pub const MAX_CHARS: usize = 200_000;
pub const EXTENSIONS: &[&str] = &["pdf", "docx", "txt", "md", "markdown"];

pub fn is_document(ext: &str) -> bool {
    EXTENSIONS.contains(&ext)
}

/// A document, or an image whose text can be read (a scan, a photo of a
/// receipt, a screenshot).
pub fn is_readable(ext: &str) -> bool {
    is_document(ext) || ocr::is_image(ext)
}

/// Extracts readable text. The caller has already checked `path` is inside a
/// granted folder.
pub fn extract_text(path: &Path) -> Result<String> {
    extract(path, ocr::MAX_PDF_PAGES)
}

/// The opening of a file: enough to tell what it is. Scanned PDFs are read
/// only up to their second page, so this stays quick.
pub fn extract_start(path: &Path) -> Result<String> {
    extract(path, 2)
}

fn extract(path: &Path, scanned_pages: usize) -> Result<String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let text = match ext.as_str() {
        "pdf" => pdf(path, scanned_pages)?,
        "docx" => docx(path)?,
        "txt" | "md" | "markdown" => String::from_utf8_lossy(&std::fs::read(path)?).into_owned(),
        e if ocr::is_image(e) => ocr::image_text(path)?,
        _ => return Err(AppError::Invalid(format!("{ext} files aren't supported yet"))),
    };
    let text = tidy(&text);
    if text.trim().is_empty() {
        return Err(AppError::Invalid("no readable text found in this file".into()));
    }
    Ok(text.chars().take(MAX_CHARS).collect())
}

fn pdf(path: &Path, scanned_pages: usize) -> Result<String> {
    // Apple's PDFKit (the engine behind Preview) reads every page reliably;
    // the pure-Rust reader is the fallback.
    #[cfg(target_os = "macos")]
    if let Some(text) = pdfkit_text(path).filter(|t| has_text(t)) {
        return Ok(text);
    }
    let owned = path.to_path_buf();
    // The PDF parser can panic on malformed files; contain it to this file.
    let text = std::panic::catch_unwind(move || pdf_extract::extract_text(&owned))
        .map_err(|_| AppError::Invalid("this PDF couldn't be read".into()))?
        .map_err(|e| AppError::Invalid(format!("this PDF couldn't be read: {e}")));
    match text {
        Ok(t) if has_text(&t) => Ok(t),
        // No text layer: a scan. Read the pages themselves.
        _ => ocr::pdf_pages(path, scanned_pages),
    }
}

#[cfg(target_os = "macos")]
fn pdfkit_text(path: &Path) -> Option<String> {
    use objc2::AllocAnyThread;
    use objc2_foundation::{NSString, NSURL};
    use objc2_pdf_kit::PDFDocument;
    let url = NSURL::fileURLWithPath(&NSString::from_str(path.to_str()?));
    // SAFETY: plain PDFKit calls on a local file URL; PDFDocument is safe to use off the main thread.
    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) }?;
    let text = unsafe { doc.string() }?;
    Some(text.to_string())
}

/// More than a stray page number or header: a real text layer.
fn has_text(t: &str) -> bool {
    t.chars().filter(|c| c.is_alphanumeric()).count() >= 20
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
