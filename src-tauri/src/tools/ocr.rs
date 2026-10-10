//! Reads the text in images and scanned PDFs with Apple's Vision framework
//! (the engine behind Live Text). Runs entirely on the Mac, needs no download.

use std::path::Path;

use crate::error::{AppError, Result};

/// Image formats Vision can read text from.
pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "heic", "tif", "tiff", "gif", "bmp", "webp"];

/// Scanned PDFs are read up to this many pages; enough to name or summarize
/// them without minutes of waiting.
pub const MAX_PDF_PAGES: usize = 12;

pub fn is_image(ext: &str) -> bool {
    IMAGE_EXTENSIONS.contains(&ext)
}

/// The text in an image, line by line.
pub fn image_text(path: &Path) -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        mac::image_text(path)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err(AppError::Invalid("reading text in images needs macOS".into()))
    }
}

/// The text on the first `pages` pages of a scanned PDF (one with no text layer).
pub fn pdf_pages(path: &Path, pages: usize) -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        mac::pdf_text(path, pages)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (path, pages);
        Err(AppError::Invalid("reading scanned PDFs needs macOS".into()))
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use std::path::Path;
    use std::ptr;

    use objc2::rc::Retained;
    use objc2::AllocAnyThread;
    use objc2_core_foundation::{CFRetained, CFString, CFURLPathStyle, CGPoint, CGRect, CGSize, CFURL};
    use objc2_core_graphics::{
        CGBitmapContextCreate, CGBitmapContextCreateImage, CGColorSpace, CGContext, CGImage, CGPDFBox,
        CGPDFDocument, CGPDFPage,
    };
    use objc2_foundation::{NSArray, NSDictionary, NSString, NSURL};
    use objc2_image_io::CGImageSource;
    use objc2_vision::{
        VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
    };

    use super::{AppError, Result};

    /// Longest side of a rendered PDF page. About 200 dpi for A4: sharp
    /// enough for small print, small enough to stay fast.
    const PAGE_PIXELS: f64 = 2400.0;

    fn request() -> Retained<VNRecognizeTextRequest> {
        let req = VNRecognizeTextRequest::new();
        req.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
        req.setUsesLanguageCorrection(true);
        req.setAutomaticallyDetectsLanguage(true);
        req
    }

    fn lines(req: &VNRecognizeTextRequest) -> String {
        let mut out = Vec::new();
        for obs in req.results().iter().flat_map(|r| r.iter()) {
            if let Some(best) = obs.topCandidates(1).firstObject() {
                out.push(best.string().to_string());
            }
        }
        out.join("\n")
    }

    fn run(handler: &VNImageRequestHandler, req: &Retained<VNRecognizeTextRequest>) -> Result<String> {
        let as_request: &VNRequest = req;
        let requests = NSArray::from_slice(&[as_request]);
        handler
            .performRequests_error(&requests)
            .map_err(|e| AppError::Invalid(format!("text recognition failed: {}", e.localizedDescription())))?;
        Ok(lines(req))
    }

    pub fn image_text(path: &Path) -> Result<String> {
        let path = path.to_str().ok_or_else(|| AppError::Invalid("unsupported file name".into()))?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path));
        let options = NSDictionary::new();
        // SAFETY: a local file URL and an empty options dictionary.
        let handler = unsafe { VNImageRequestHandler::initWithURL_options(VNImageRequestHandler::alloc(), &url, &options) };
        let text = run(&handler, &request())?;
        if text.chars().filter(|c| c.is_alphanumeric()).count() >= 20 {
            return Ok(text);
        }
        // Dark text on a transparent background reads as black on black:
        // flatten the image onto white and look again.
        match flattened(path) {
            Some(image) => {
                // SAFETY: a CGImage we just drew and an empty options dictionary.
                let handler =
                    unsafe { VNImageRequestHandler::initWithCGImage_options(VNImageRequestHandler::alloc(), &image, &options) };
                let second = run(&handler, &request())?;
                Ok(if second.len() > text.len() { second } else { text })
            }
            None => Ok(text),
        }
    }

    /// The image at `path` drawn onto a white, grayscale canvas.
    fn flattened(path: &str) -> Option<CFRetained<CGImage>> {
        let url = CFURL::with_file_system_path(None, Some(&CFString::from_str(path)), CFURLPathStyle::CFURLPOSIXPathStyle, false)?;
        // SAFETY: a local file URL, no options.
        let source = unsafe { CGImageSource::with_url(&url, None) }?;
        // SAFETY: the first image of the source, no options.
        let image = unsafe { source.image_at_index(0, None) }?;
        let (w, h) = (CGImage::width(Some(&image)) as f64, CGImage::height(Some(&image)) as f64);
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let scale = (PAGE_PIXELS * 1.5 / w.max(h)).min(1.0);
        let (pw, ph) = ((w * scale).round() as usize, (h * scale).round() as usize);
        let canvas = white_canvas(pw, ph)?;
        CGContext::draw_image(Some(&canvas), CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(pw as f64, ph as f64)), Some(&image));
        CGBitmapContextCreateImage(Some(&canvas))
    }

    /// An 8-bit grayscale bitmap filled with white.
    fn white_canvas(w: usize, h: usize) -> Option<CFRetained<CGContext>> {
        let space = CGColorSpace::new_device_gray()?;
        // SAFETY: a null data pointer lets CoreGraphics own the buffer;
        // 8-bit gray with no alpha (bitmap info 0) is a supported format.
        let ctx = unsafe { CGBitmapContextCreate(ptr::null_mut(), w, h, 8, 0, Some(&space), 0) }?;
        CGContext::set_gray_fill_color(Some(&ctx), 1.0, 1.0);
        CGContext::fill_rect(Some(&ctx), CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(w as f64, h as f64)));
        Some(ctx)
    }

    /// Renders a PDF page to a white, grayscale bitmap.
    fn render(page: &CGPDFPage) -> Option<CFRetained<CGImage>> {
        let rect: CGRect = CGPDFPage::box_rect(Some(page), CGPDFBox::MediaBox);
        let (w, h) = (rect.size.width, rect.size.height);
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let scale = (PAGE_PIXELS / w.max(h)).min(4.0);
        let (pw, ph) = ((w * scale).round() as usize, (h * scale).round() as usize);
        let ctx = white_canvas(pw, ph)?;
        CGContext::scale_ctm(Some(&ctx), scale, scale);
        CGContext::translate_ctm(Some(&ctx), -rect.origin.x, -rect.origin.y);
        CGContext::draw_pdf_page(Some(&ctx), Some(page));
        CGBitmapContextCreateImage(Some(&ctx))
    }

    pub fn pdf_text(path: &Path, max_pages: usize) -> Result<String> {
        let unreadable = || AppError::Invalid("this PDF couldn't be opened".into());
        let path = path.to_str().ok_or_else(unreadable)?;
        let url = CFURL::with_file_system_path(None, Some(&CFString::from_str(path)), CFURLPathStyle::CFURLPOSIXPathStyle, false)
            .ok_or_else(unreadable)?;
        let doc = CGPDFDocument::with_url(Some(&url)).ok_or_else(unreadable)?;
        let pages = CGPDFDocument::number_of_pages(Some(&doc)).min(max_pages);
        let mut text = Vec::new();
        for n in 1..=pages {
            let Some(page) = CGPDFDocument::page(Some(&doc), n) else { continue };
            let Some(image) = render(&page) else { continue };
            let options = NSDictionary::new();
            // SAFETY: a CGImage we just rendered and an empty options dictionary.
            let handler =
                unsafe { VNImageRequestHandler::initWithCGImage_options(VNImageRequestHandler::alloc(), &image, &options) };
            let page_text = run(&handler, &request())?;
            if !page_text.trim().is_empty() {
                text.push(page_text);
            }
        }
        Ok(text.join("\n\n"))
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::tools::pdf::{render, Block, PdfDoc};
    use std::process::Command;

    /// A real PDF with text, and the same page as a PNG (a "scan" with no
    /// text layer), made with macOS's own `sips`.
    fn scan_fixture(dir: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
        let doc = PdfDoc {
            title: "Invoice from Acme Corporation".into(),
            subtitle: "Issued 14 March 2026".into(),
            blocks: vec![Block::Paragraph("Total amount due: 1,250 dollars for website design services.".into())],
        };
        let pdf = dir.join("text.pdf");
        std::fs::write(&pdf, render(&doc).unwrap()).unwrap();
        let png = dir.join("scan_0034.png");
        let ok = Command::new("sips")
            .args(["-s", "format", "png", "-Z", "2000"])
            .arg(&pdf)
            .arg("--out")
            .arg(&png)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "sips converts the PDF page to an image");
        (pdf, png)
    }

    #[test]
    fn reads_text_in_images_and_scanned_pdfs() {
        let dir = tempfile::tempdir().unwrap();
        let (pdf, png) = scan_fixture(dir.path());

        let from_image = image_text(&png).unwrap();
        assert!(from_image.contains("Acme Corporation"), "{from_image}");
        assert!(from_image.contains("1,250"), "{from_image}");

        // Rendering ignores the text layer, so this exercises the scan path.
        let from_pdf = pdf_pages(&pdf, MAX_PDF_PAGES).unwrap();
        assert!(from_pdf.contains("Invoice from Acme"), "{from_pdf}");

        // A PNG with a transparent background (dark text on nothing) still reads.
        let txt = dir.path().join("receipt.txt");
        std::fs::write(&txt, "CORNER CAFE\nReceipt 2026-05-02\nFlat white 4.50\nTotal 4.50\n").unwrap();
        let out = Command::new("cupsfilter").arg(&txt).output().unwrap();
        assert!(out.status.success());
        let receipt_pdf = dir.path().join("receipt.pdf");
        std::fs::write(&receipt_pdf, out.stdout).unwrap();
        let clear = dir.path().join("IMG_9.png");
        Command::new("sips").args(["-s", "format", "png", "-Z", "1800"]).arg(&receipt_pdf).arg("--out").arg(&clear).output().unwrap();
        let alpha = Command::new("sips").args(["-g", "hasAlpha"]).arg(&clear).output().unwrap();
        assert!(String::from_utf8_lossy(&alpha.stdout).contains("yes"), "the fixture really is transparent");
        let from_clear = image_text(&clear).unwrap();
        assert!(from_clear.contains("CORNER CAFE"), "{from_clear}");

        // Through the normal document reader, an image is just another document.
        let via_reader = crate::tools::documents::extract_text(&png).unwrap();
        assert!(via_reader.contains("Acme"));
    }
}
