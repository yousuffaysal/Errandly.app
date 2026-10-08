//! Spreadsheet reading and deterministic analysis (PRD §13). All arithmetic is
//! done here, never by the model (§13.5).

use std::collections::BTreeMap;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

pub const EXTENSIONS: &[&str] = &["xlsx", "xls", "xlsm", "ods", "csv"];
const MAX_ROWS: usize = 100_000;
const MAX_GROUPS: usize = 12;

pub fn is_spreadsheet(ext: &str) -> bool {
    EXTENSIONS.contains(&ext)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Number(f64),
    Text(String),
    Empty,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
    pub sheet: Option<String>,
}

/// Reads the first sheet (or the CSV) with the first row as headers.
pub fn read(path: &Path) -> Result<Table> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let (sheet, grid) = if ext == "csv" { (None, read_csv(path)?) } else { read_workbook(path)? };
    let mut rows = grid.into_iter().skip_while(|r| r.iter().all(|c| *c == Cell::Empty));
    let header_row = rows.next().ok_or_else(|| AppError::Invalid("this spreadsheet is empty".into()))?;
    let headers: Vec<String> = header_row
        .iter()
        .enumerate()
        .map(|(i, c)| match c {
            Cell::Text(t) if !t.trim().is_empty() => t.trim().to_string(),
            Cell::Number(n) => format_number(*n),
            _ => format!("Column {}", i + 1),
        })
        .collect();
    let width = headers.len();
    let rows: Vec<Vec<Cell>> = rows
        .filter(|r| r.iter().any(|c| *c != Cell::Empty))
        .take(MAX_ROWS)
        .map(|mut r| {
            r.resize(width, Cell::Empty);
            r
        })
        .collect();
    Ok(Table { headers, rows, sheet })
}

fn read_csv(path: &Path) -> Result<Vec<Vec<Cell>>> {
    let mut reader = csv::ReaderBuilder::new().has_headers(false).flexible(true).from_path(path)
        .map_err(|e| AppError::Invalid(format!("this CSV couldn't be read: {e}")))?;
    let mut grid = Vec::new();
    for record in reader.records().take(MAX_ROWS + 1) {
        let record = record.map_err(|e| AppError::Invalid(format!("this CSV couldn't be read: {e}")))?;
        grid.push(record.iter().map(parse_cell).collect());
    }
    Ok(grid)
}

fn read_workbook(path: &Path) -> Result<(Option<String>, Vec<Vec<Cell>>)> {
    let mut wb = open_workbook_auto(path).map_err(|e| AppError::Invalid(format!("this spreadsheet couldn't be opened: {e}")))?;
    let name = wb.sheet_names().first().cloned().ok_or_else(|| AppError::Invalid("this workbook has no sheets".into()))?;
    let range = wb.worksheet_range(&name).map_err(|e| AppError::Invalid(format!("couldn't read sheet {name}: {e}")))?;
    let grid = range
        .rows()
        .take(MAX_ROWS + 1)
        .map(|r| {
            r.iter()
                .map(|d| match d {
                    Data::Int(i) => Cell::Number(*i as f64),
                    Data::Float(f) => Cell::Number(*f),
                    Data::String(s) => parse_cell(s),
                    Data::Empty => Cell::Empty,
                    other => Cell::Text(other.to_string()),
                })
                .collect()
        })
        .collect();
    Ok((Some(name), grid))
}

/// Text that is really a number ("$1,250.00", "৳ 900", "(45)") becomes a number.
fn parse_cell(raw: &str) -> Cell {
    let t = raw.trim();
    if t.is_empty() {
        return Cell::Empty;
    }
    let negative = t.starts_with('(') && t.ends_with(')') || t.starts_with('-');
    let digits: String = t.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    let only_number_chars = t.chars().all(|c| c.is_ascii_digit() || " .,-()$€£¥৳₹%+".contains(c));
    match (only_number_chars && !digits.is_empty(), digits.parse::<f64>()) {
        (true, Ok(n)) => Cell::Number(if negative { -n } else { n }),
        _ => Cell::Text(t.to_string()),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ColumnStats {
    pub name: String,
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    pub total: f64,
    pub rows: usize,
    pub share: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub rows: usize,
    pub numeric: Vec<ColumnStats>,
    pub value_column: Option<String>,
    pub group_column: Option<String>,
    pub total: Option<f64>,
    pub groups: Vec<Group>,
    /// Groups beyond the top ones, summed into "Everything else".
    pub other_groups: usize,
    /// Rows whose value cell wasn't a number and so were left out of totals.
    pub skipped_rows: usize,
}

/// Columns where at least 75% of the filled cells are numbers (a few notes
/// like "n/a" are common in real sheets).
pub fn numeric_columns(t: &Table) -> Vec<usize> {
    (0..t.headers.len())
        .filter(|&i| {
            let (mut nums, mut filled) = (0, 0);
            for r in &t.rows {
                match &r[i] {
                    Cell::Number(_) => {
                        nums += 1;
                        filled += 1
                    }
                    Cell::Text(_) => filled += 1,
                    Cell::Empty => {}
                }
            }
            filled > 0 && nums * 4 >= filled * 3
        })
        .collect()
}

/// Text columns that look like categories: 2 to 200 distinct values.
pub fn category_columns(t: &Table) -> Vec<usize> {
    let numeric = numeric_columns(t);
    (0..t.headers.len())
        .filter(|i| !numeric.contains(i))
        .filter(|&i| {
            let mut seen = std::collections::HashSet::new();
            for r in &t.rows {
                if let Cell::Text(s) = &r[i] {
                    seen.insert(s.to_lowercase());
                    if seen.len() > 200 {
                        return false;
                    }
                }
            }
            seen.len() >= 2
        })
        .collect()
}

/// Totals `value` (a numeric column) overall and per `group` (a category column).
pub fn analyze(t: &Table, value: Option<usize>, group: Option<usize>) -> Analysis {
    let numeric: Vec<ColumnStats> = numeric_columns(t)
        .into_iter()
        .map(|i| {
            let vals: Vec<f64> = t.rows.iter().filter_map(|r| if let Cell::Number(n) = r[i] { Some(n) } else { None }).collect();
            let sum: f64 = vals.iter().sum();
            ColumnStats {
                name: t.headers[i].clone(),
                count: vals.len(),
                sum: round2(sum),
                mean: round2(if vals.is_empty() { 0.0 } else { sum / vals.len() as f64 }),
                min: round2(vals.iter().cloned().fold(f64::INFINITY, f64::min)),
                max: round2(vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max)),
            }
        })
        .collect();

    let mut analysis = Analysis {
        rows: t.rows.len(),
        numeric,
        value_column: value.map(|i| t.headers[i].clone()),
        group_column: group.map(|i| t.headers[i].clone()),
        total: None,
        groups: vec![],
        other_groups: 0,
        skipped_rows: 0,
    };
    let Some(v) = value else { return analysis };

    let mut total = 0.0;
    let mut by: BTreeMap<String, (String, f64, usize)> = BTreeMap::new();
    for r in &t.rows {
        let Cell::Number(n) = r[v] else {
            analysis.skipped_rows += 1;
            continue;
        };
        total += n;
        if let Some(g) = group {
            let label = match &r[g] {
                Cell::Text(s) => s.clone(),
                Cell::Number(n) => format_number(*n),
                Cell::Empty => "(blank)".into(),
            };
            let e = by.entry(label.to_lowercase()).or_insert((label, 0.0, 0));
            e.1 += n;
            e.2 += 1;
        }
    }
    analysis.total = Some(round2(total));
    let mut groups: Vec<Group> = by
        .into_values()
        .map(|(name, sum, rows)| Group { name, total: round2(sum), rows, share: 0.0 })
        .collect();
    groups.sort_by(|a, b| b.total.abs().total_cmp(&a.total.abs()));
    if groups.len() > MAX_GROUPS {
        let rest = groups.split_off(MAX_GROUPS - 1);
        analysis.other_groups = rest.len();
        groups.push(Group {
            name: "Everything else".into(),
            total: round2(rest.iter().map(|g| g.total).sum()),
            rows: rest.iter().map(|g| g.rows).sum(),
            share: 0.0,
        });
    }
    for g in &mut groups {
        g.share = if total.abs() > f64::EPSILON { round2(g.total / total * 100.0) } else { 0.0 };
    }
    analysis.groups = groups;
    analysis
}

fn round2(n: f64) -> f64 {
    if n.is_finite() { (n * 100.0).round() / 100.0 } else { 0.0 }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 { format!("{}", n as i64) } else { format!("{n}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn csv(content: &str) -> Table {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("sales.csv");
        std::fs::write(&p, content).unwrap();
        read(&p).unwrap()
    }

    #[test]
    fn parses_money_text_as_numbers() {
        assert_eq!(parse_cell("$1,250.50"), Cell::Number(1250.5));
        assert_eq!(parse_cell("(45)"), Cell::Number(-45.0));
        assert_eq!(parse_cell("৳ 900"), Cell::Number(900.0));
        assert_eq!(parse_cell("INV-2024"), Cell::Text("INV-2024".into()));
        assert_eq!(parse_cell("  "), Cell::Empty);
    }

    /// The exported numbers must match a hand calculation exactly (PRD §13.5).
    #[test]
    fn totals_and_groups_are_exact() {
        let t = csv("Product,Region,Revenue\nTea,North,$100.10\nCoffee,South,200.20\nTea,South,\"1,000\"\nJuice,North,n/a\n");
        assert_eq!(t.headers, ["Product", "Region", "Revenue"]);
        let value = numeric_columns(&t)[0];
        assert_eq!(t.headers[value], "Revenue");
        assert_eq!(category_columns(&t), vec![0, 1]);

        let a = analyze(&t, Some(value), Some(0));
        assert_eq!(a.total, Some(1300.3));
        assert_eq!(a.skipped_rows, 1, "the n/a row is left out, not guessed");
        assert_eq!(a.groups[0], Group { name: "Tea".into(), total: 1100.1, rows: 2, share: 84.6 });
        assert_eq!(a.groups[1].name, "Coffee");
        let revenue = a.numeric.iter().find(|c| c.name == "Revenue").unwrap();
        assert_eq!((revenue.count, revenue.sum, revenue.min, revenue.max), (3, 1300.3, 100.1, 1000.0));
    }

    #[test]
    fn many_groups_are_capped() {
        let mut s = String::from("Item,Cost\n");
        for i in 0..30 {
            s.push_str(&format!("item{i},{}\n", i + 1));
        }
        let t = csv(&s);
        let a = analyze(&t, Some(1), Some(0));
        assert_eq!(a.groups.len(), MAX_GROUPS);
        assert_eq!(a.groups.last().unwrap().name, "Everything else");
        assert_eq!(a.groups.iter().map(|g| g.total).sum::<f64>(), a.total.unwrap());
    }
}
