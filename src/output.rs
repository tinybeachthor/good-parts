use std::io::{self, Write};

use arborist::FileReport;
use clap::ValueEnum;
use serde::Serialize;

use crate::report::{FileHotspot, FunctionHotspot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Table,
    Json,
}

/// Totals across the whole scan, independent of filtering and truncation.
#[derive(Debug, Serialize)]
pub struct Summary {
    pub files: usize,
    pub functions: usize,
    pub over_threshold: usize,
    pub threshold: u64,
}

impl Summary {
    pub fn new(reports: &[FileReport], threshold: u64) -> Self {
        let functions = reports.iter().flat_map(|r| &r.functions);
        Self {
            files: reports.len(),
            functions: functions.clone().count(),
            over_threshold: functions
                .filter(|f| f.exceeds_threshold == Some(true))
                .count(),
            threshold,
        }
    }
}

#[derive(Serialize)]
struct JsonReport<'a, T> {
    summary: &'a Summary,
    hotspots: &'a [T],
}

pub fn write_json<T: Serialize>(
    out: &mut impl Write,
    summary: &Summary,
    hotspots: &[T],
) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *out, &JsonReport { summary, hotspots })?;
    writeln!(out)
}

fn write_summary(out: &mut impl Write, summary: &Summary) -> io::Result<()> {
    writeln!(
        out,
        "\n{} files, {} functions, {} over cognitive threshold {}",
        summary.files, summary.functions, summary.over_threshold, summary.threshold
    )
}

/// Render rows as left-aligned columns separated by two spaces.
fn write_table(out: &mut impl Write, rows: &[Vec<String>]) -> io::Result<()> {
    let columns = rows.first().map_or(0, Vec::len);
    let widths: Vec<usize> = (0..columns)
        .map(|c| rows.iter().map(|r| r[c].len()).max().unwrap_or(0))
        .collect();
    for row in rows {
        let line: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(cell, &w)| format!("{cell:<w$}"))
            .collect();
        writeln!(out, "{}", line.join("  ").trim_end())?;
    }
    Ok(())
}

fn flag(over: bool) -> &'static str {
    if over { "!" } else { "" }
}

pub fn write_function_table(
    out: &mut impl Write,
    summary: &Summary,
    hotspots: &[FunctionHotspot],
) -> io::Result<()> {
    let header = ["SCORE", "COG", "CYC", "SLOC", "LOCATION", "FUNCTION"];
    let mut rows = vec![header.map(String::from).to_vec()];
    rows.extend(hotspots.iter().map(|h| {
        vec![
            format!("{:.1}", h.score),
            format!("{}{}", h.cognitive, flag(h.exceeds_threshold)),
            h.cyclomatic.to_string(),
            h.sloc.to_string(),
            format!("{}:{}", h.file, h.start_line),
            h.name.clone(),
        ]
    }));
    write_table(out, &rows)?;
    write_summary(out, summary)
}

pub fn write_file_table(
    out: &mut impl Write,
    summary: &Summary,
    hotspots: &[FileHotspot],
) -> io::Result<()> {
    let header = ["SCORE", "COG", "MAX", "FNS", "OVER", "SLOC", "FILE"];
    let mut rows = vec![header.map(String::from).to_vec()];
    rows.extend(hotspots.iter().map(|h| {
        vec![
            format!("{:.1}", h.score),
            h.total_cognitive.to_string(),
            format!("{}{}", h.max_cognitive, flag(h.over_threshold > 0)),
            h.functions.to_string(),
            h.over_threshold.to_string(),
            h.sloc.to_string(),
            h.file.clone(),
        ]
    }));
    write_table(out, &rows)?;
    write_summary(out, summary)
}
