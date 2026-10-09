//! Turns Errandly's results into PDFs: document summaries, spreadsheet
//! reports, and plain answers (emails, plans, study notes).

use super::analyst::SheetReport;
use super::documents::DocReport;
use crate::tools::pdf::{from_markdown, Block, PdfDoc};

/// 1234567.5 → "1,234,567.50".
pub fn thousands(n: f64) -> String {
    let s = format!("{:.2}", n.abs());
    let (int, frac) = s.split_once('.').unwrap_or((&s, "00"));
    let mut out = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{}{out}.{frac}", if n < 0.0 { "-" } else { "" })
}

pub fn documents(report: &DocReport, source: &str) -> PdfDoc {
    let n = report.documents.len();
    let mut blocks = Vec::new();
    if let Some(o) = &report.overview {
        blocks.push(Block::Heading("Overview".into()));
        blocks.push(Block::Paragraph(o.clone()));
    }
    for d in &report.documents {
        blocks.push(Block::Heading(d.name.clone()));
        blocks.push(Block::Paragraph(d.summary.clone()));
        if !d.key_points.is_empty() {
            blocks.push(Block::Bullets(d.key_points.clone()));
        }
        if d.truncated {
            blocks.push(Block::Note("This document is long; only its first part was summarized.".into()));
        }
        if d.flagged {
            blocks.push(Block::Note(
                "This document contained instructions aimed at AI assistants. Errandly removed and ignored them; \
                 double-check anything surprising."
                    .into(),
            ));
        }
    }
    if !report.unreadable.is_empty() {
        blocks.push(Block::Heading("Couldn't read".into()));
        blocks.push(Block::Bullets(report.unreadable.iter().map(|(name, why)| format!("{name}: {why}")).collect()));
    }
    PdfDoc {
        title: if n == 1 { format!("Summary of {}", report.documents[0].name) } else { "Document summaries".into() },
        subtitle: format!("{source} · {n} document{}", if n == 1 { "" } else { "s" }),
        blocks,
    }
}

pub fn spreadsheet(r: &SheetReport) -> PdfDoc {
    let a = &r.analysis;
    let mut blocks = Vec::new();
    if let (Some(col), Some(total)) = (&a.value_column, a.total) {
        blocks.push(Block::Paragraph(format!("Total **{col}**: {}", thousands(total))));
    }
    if !r.insights.is_empty() {
        blocks.push(Block::Heading("What stands out".into()));
        blocks.push(Block::Bullets(r.insights.clone()));
    }
    if !a.groups.is_empty() {
        blocks.push(Block::Heading(format!("By {}", a.group_column.as_deref().unwrap_or("group"))));
        blocks.push(Block::Table {
            headers: vec![
                a.group_column.clone().unwrap_or_else(|| "Group".into()),
                a.value_column.clone().unwrap_or_else(|| "Total".into()),
                "Share".into(),
                "Rows".into(),
            ],
            rows: a
                .groups
                .iter()
                .map(|g| vec![g.name.clone(), thousands(g.total), format!("{:.1}%", g.share), g.rows.to_string()])
                .collect(),
            weights: vec![3, 2, 1, 1],
        });
        if a.other_groups > 0 {
            blocks.push(Block::Note(format!("{} smaller groups are combined into the last row.", a.other_groups)));
        }
    }
    if !a.numeric.is_empty() {
        blocks.push(Block::Heading("Number columns".into()));
        blocks.push(Block::Table {
            headers: ["Column", "Sum", "Average", "Min", "Max"].map(String::from).to_vec(),
            rows: a
                .numeric
                .iter()
                .map(|s| vec![s.name.clone(), thousands(s.sum), thousands(s.mean), thousands(s.min), thousands(s.max)])
                .collect(),
            weights: vec![3, 2, 2, 2, 2],
        });
    }
    if a.skipped_rows > 0 {
        blocks.push(Block::Note(format!("{} rows had no number in the value column and were left out of the totals.", a.skipped_rows)));
    }
    blocks.push(Block::Note("Every number was calculated directly from the file by Errandly.".into()));
    PdfDoc {
        title: r.title.clone(),
        subtitle: format!("{}{} · {} rows", r.file, r.sheet.as_deref().map(|s| format!(" ({s})")).unwrap_or_default(), a.rows),
        blocks,
    }
}

/// A plain answer. The title is the first heading if it starts with one,
/// else the conversation title.
pub fn answer(text: &str, conversation_title: &str) -> PdfDoc {
    let mut blocks = from_markdown(text);
    let title = match blocks.first() {
        Some(Block::Heading(h)) => {
            let h = h.clone();
            blocks.remove(0);
            h
        }
        _ => conversation_title.to_string(),
    };
    PdfDoc { title, subtitle: "Written by Errandly".into(), blocks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_titles() {
        assert_eq!(thousands(7935.75), "7,935.75");
        assert_eq!(thousands(-1234567.5), "-1,234,567.50");
        assert_eq!(thousands(12.0), "12.00");
        let doc = answer("# Study plan\n\n1. Read\n2. Practice", "Chat");
        assert_eq!(doc.title, "Study plan");
        assert_eq!(doc.blocks, vec![Block::Numbered(vec!["Read".into(), "Practice".into()])]);
        assert_eq!(answer("Dear Sam,\nThanks.", "Email to Sam").title, "Email to Sam");
        assert!(crate::tools::pdf::render(&doc).unwrap().starts_with(b"%PDF"));
    }
}
