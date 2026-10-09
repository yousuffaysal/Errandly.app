//! Clean, readable PDFs made on the Mac, set in Georgia (the app's heading
//! face) with lists, tables and page numbers. Uses a font every Mac already
//! has, so no font files ship with the app.

use genpdf::elements::{Break, FrameCellDecorator, OrderedList, Paragraph, TableLayout, UnorderedList};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Document, Element, SimplePageDecorator};

use crate::error::{AppError, Result};

const FONT_DIR: &str = "/System/Library/Fonts/Supplemental";
const INK: Color = Color::Rgb(31, 36, 27);
const MUTED: Color = Color::Rgb(95, 102, 85);
const OLIVE: Color = Color::Rgb(74, 90, 53);

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading(String),
    Paragraph(String),
    Bullets(Vec<String>),
    Numbered(Vec<String>),
    Table { headers: Vec<String>, rows: Vec<Vec<String>>, weights: Vec<usize> },
    /// Small grey text, e.g. a caveat.
    Note(String),
}

pub struct PdfDoc {
    pub title: String,
    pub subtitle: String,
    pub blocks: Vec<Block>,
}

fn family(name: &str) -> Result<FontFamily<FontData>> {
    let load = |suffix: &str| {
        let file = if suffix.is_empty() { format!("{FONT_DIR}/{name}.ttf") } else { format!("{FONT_DIR}/{name} {suffix}.ttf") };
        FontData::load(&file, None).map_err(|e| AppError::Invalid(format!("couldn't load the {name} font: {e}")))
    };
    Ok(FontFamily { regular: load("")?, bold: load("Bold")?, italic: load("Italic")?, bold_italic: load("Bold Italic")? })
}

/// Drops characters the fonts can't draw (emoji, pictographs), so the PDF
/// never shows empty boxes.
fn printable(s: &str) -> String {
    s.chars()
        .filter(|c| {
            let u = *c as u32;
            !(u >= 0x1F000 || (0x2600..=0x27BF).contains(&u) || (0xFE00..=0xFE0F).contains(&u) || u == 0x200D)
        })
        .collect::<String>()
        .replace('\t', "    ")
}

/// A paragraph with **bold** spans rendered in bold.
fn rich(text: &str, style: Style) -> Paragraph {
    let mut p = Paragraph::default();
    for (i, part) in printable(text).split("**").enumerate() {
        if part.is_empty() {
            continue;
        }
        if i % 2 == 1 { p.push_styled(part.to_string(), style.bold()) } else { p.push_styled(part.to_string(), style) }
    }
    p
}

pub fn render(doc: &PdfDoc) -> Result<Vec<u8>> {
    // One family keeps files small: every font is embedded whole.
    let mut pdf = Document::new(family("Georgia")?);
    pdf.set_title(printable(&doc.title));
    pdf.set_paper_size(genpdf::PaperSize::A4);
    pdf.set_font_size(11);
    pdf.set_line_spacing(1.35);

    let mut deco = SimplePageDecorator::new();
    deco.set_margins(18);
    let running = printable(&doc.title);
    deco.set_header(move |page| {
        let label = if page == 1 { String::new() } else { format!("{running}  ·  page {page}") };
        Paragraph::new(label)
            .aligned(Alignment::Right)
            .styled(Style::new().with_font_size(8).with_color(MUTED))
            .padded(genpdf::Margins::trbl(0, 0, 4, 0))
    });
    pdf.set_page_decorator(deco);

    let body = Style::new().with_color(INK);
    pdf.push(rich(&doc.title, Style::new().with_font_size(22).with_color(INK)));
    pdf.push(Paragraph::new(printable(&doc.subtitle)).styled(Style::new().with_font_size(9).with_color(MUTED)));
    pdf.push(Break::new(1.2));

    for block in &doc.blocks {
        match block {
            Block::Heading(h) => {
                pdf.push(Break::new(0.6));
                pdf.push(rich(h, Style::new().with_font_size(15).with_color(OLIVE)));
                pdf.push(Break::new(0.2));
            }
            Block::Paragraph(t) => {
                pdf.push(rich(t, body));
                pdf.push(Break::new(0.5));
            }
            Block::Bullets(items) => {
                let mut list = UnorderedList::new();
                for it in items {
                    list.push(rich(it, body));
                }
                pdf.push(list);
                pdf.push(Break::new(0.5));
            }
            Block::Numbered(items) => {
                let mut list = OrderedList::new();
                for it in items {
                    list.push(rich(it, body));
                }
                pdf.push(list);
                pdf.push(Break::new(0.5));
            }
            Block::Table { headers, rows, weights } => {
                let mut table = TableLayout::new(weights.clone());
                table.set_cell_decorator(FrameCellDecorator::new(true, true, false));
                let mut head = table.row();
                for h in headers {
                    head.push_element(rich(h, body.bold().with_font_size(10)).padded(1));
                }
                head.push().map_err(|e| AppError::Invalid(e.to_string()))?;
                for r in rows {
                    let mut row = table.row();
                    for (i, cell) in r.iter().enumerate() {
                        let p = rich(cell, body.with_font_size(10));
                        let p = if i > 0 { p.aligned(Alignment::Right) } else { p };
                        row.push_element(p.padded(1));
                    }
                    row.push().map_err(|e| AppError::Invalid(e.to_string()))?;
                }
                pdf.push(table);
                pdf.push(Break::new(0.6));
            }
            Block::Note(t) => {
                pdf.push(rich(t, Style::new().with_font_size(9).with_color(MUTED).italic()));
                pdf.push(Break::new(0.4));
            }
        }
    }
    pdf.push(Break::new(1.0));
    pdf.push(Paragraph::new("Made with Errandly on this Mac").styled(Style::new().with_font_size(8).with_color(MUTED)));

    let mut out = Vec::new();
    pdf.render(&mut out).map_err(|e| AppError::Invalid(format!("couldn't create the PDF: {e}")))?;
    Ok(out)
}

/// Turns an assistant reply (light Markdown) into PDF blocks: headings,
/// bullet and numbered lists, paragraphs, keeping **bold**.
pub fn from_markdown(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let flush = |para: &mut Vec<String>, blocks: &mut Vec<Block>| {
        if !para.is_empty() {
            blocks.push(Block::Paragraph(para.join(" ")));
            para.clear();
        }
    };
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            flush(&mut para, &mut blocks);
            continue;
        }
        let bullet = ["- ", "* ", "• "].iter().find_map(|p| line.strip_prefix(p));
        let numbered = line
            .split_once(". ")
            .filter(|(n, _)| !n.is_empty() && n.len() <= 3 && n.chars().all(|c| c.is_ascii_digit()))
            .map(|(_, rest)| rest);
        if let Some(h) = line.strip_prefix('#') {
            flush(&mut para, &mut blocks);
            blocks.push(Block::Heading(h.trim_start_matches('#').trim().to_string()));
        } else if let Some(item) = bullet {
            flush(&mut para, &mut blocks);
            match blocks.last_mut() {
                Some(Block::Bullets(items)) => items.push(item.to_string()),
                _ => blocks.push(Block::Bullets(vec![item.to_string()])),
            }
        } else if let Some(item) = numbered {
            flush(&mut para, &mut blocks);
            match blocks.last_mut() {
                Some(Block::Numbered(items)) => items.push(item.to_string()),
                _ => blocks.push(Block::Numbered(vec![item.to_string()])),
            }
        } else {
            para.push(line.to_string());
        }
    }
    flush(&mut para, &mut blocks);
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_becomes_blocks() {
        let b = from_markdown("# Plan\n\nIntro line\ncontinues.\n\n1. **Day 1**: basics\n2. Day 2\n\n- tip one\n- tip two");
        assert_eq!(b[0], Block::Heading("Plan".into()));
        assert_eq!(b[1], Block::Paragraph("Intro line continues.".into()));
        assert_eq!(b[2], Block::Numbered(vec!["**Day 1**: basics".into(), "Day 2".into()]));
        assert_eq!(b[3], Block::Bullets(vec!["tip one".into(), "tip two".into()]));
        assert_eq!(printable("Done ✅ 🎉 ok"), "Done   ok");
    }

    /// A real PDF comes out, and its text can be read back.
    #[test]
    fn renders_a_readable_pdf() {
        let doc = PdfDoc {
            title: "Sales by product".into(),
            subtitle: "sales.csv · 6 rows".into(),
            blocks: vec![
                Block::Paragraph("Total **Revenue**: 7,935.75".into()),
                Block::Table {
                    headers: vec!["Product".into(), "Revenue".into()],
                    rows: vec![vec!["Coffee".into(), "4,325.50".into()], vec!["Tea".into(), "3,300.00".into()]],
                    weights: vec![2, 1],
                },
                Block::Bullets((0..60).map(|i| format!("Point number {i} with some words to wrap")).collect()),
            ],
        };
        let bytes = render(&doc).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.pdf");
        std::fs::write(&path, &bytes).unwrap();
        if let Ok(keep) = std::env::var("ERRANDLY_KEEP_PDF") {
            std::fs::write(keep, &bytes).unwrap();
        }
        let text = crate::tools::documents::extract_text(&path).unwrap();
        assert!(text.contains("Coffee") && text.contains("7,935.75"), "{text}");
        assert!(text.contains("Point number 59"), "page 2 is read too: {text}");
        assert!(bytes.len() < 2_500_000, "one embedded family keeps it small: {} bytes", bytes.len());
    }
}
