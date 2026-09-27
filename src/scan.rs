use std::path::{Path, PathBuf};

use anyhow::Context;
use arborist::{AnalysisConfig, ArboristError, FileReport};
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use rayon::prelude::*;

/// Walk `root` (respecting .gitignore and `exclude` globs) and analyze every
/// file in a supported language. Report paths are relative to `root`.
pub fn scan(
    root: &Path,
    exclude: &[String],
    config: &AnalysisConfig,
) -> anyhow::Result<Vec<FileReport>> {
    let files = collect_files(root, exclude)?;

    let mut reports: Vec<FileReport> = files
        .par_iter()
        .filter_map(
            |path| match arborist::analyze_file_with_config(path, config) {
                Ok(mut report) => {
                    let relative = path.strip_prefix(root).unwrap_or(path);
                    report.path = relative.display().to_string();
                    Some(report)
                }
                Err(
                    ArboristError::UnrecognizedExtension { .. }
                    | ArboristError::LanguageNotEnabled { .. }
                    | ArboristError::UnsupportedLanguage { .. },
                ) => None,
                Err(err) => {
                    eprintln!("warning: skipping {}: {err}", path.display());
                    None
                }
            },
        )
        .collect();

    reports.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(reports)
}

fn collect_files(root: &Path, exclude: &[String]) -> anyhow::Result<Vec<PathBuf>> {
    let mut overrides = OverrideBuilder::new(root);
    for glob in exclude {
        overrides
            .add(&format!("!{glob}"))
            .with_context(|| format!("invalid exclude pattern: {glob}"))?;
    }
    let overrides = overrides.build()?;

    let mut files = Vec::new();
    for entry in WalkBuilder::new(root).overrides(overrides).build() {
        let entry = entry?;
        if entry.file_type().is_some_and(|t| t.is_file()) {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}
