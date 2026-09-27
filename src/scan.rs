use std::path::{Path, PathBuf};

use anyhow::Context;
use arborist::{AnalysisConfig, ArboristError, FileReport};
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use rayon::prelude::*;

/// Analyze every supported file under `roots` (files or directories),
/// respecting .gitignore and `exclude` globs. Report paths are shown
/// relative to the current directory when possible.
pub fn scan(
    roots: &[PathBuf],
    exclude: &[String],
    config: &AnalysisConfig,
) -> anyhow::Result<Vec<FileReport>> {
    let mut files = Vec::new();
    for root in roots {
        files.extend(collect_files(root, exclude)?);
    }
    files.sort();
    files.dedup();

    let cwd = std::env::current_dir().ok();
    let mut reports: Vec<FileReport> = files
        .par_iter()
        .filter_map(
            |path| match arborist::analyze_file_with_config(path, config) {
                Ok(mut report) => {
                    report.path = display_path(path, cwd.as_deref());
                    Some(report)
                }
                Err(
                    ArboristError::UnrecognizedExtension { .. }
                    | ArboristError::LanguageNotEnabled { .. }
                    | ArboristError::UnsupportedLanguage { .. },
                ) => {
                    if roots.contains(path) {
                        eprintln!("warning: skipping {}: unsupported language", path.display());
                    }
                    None
                }
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

/// The path as an agent or editor in `cwd` would open it.
fn display_path(path: &Path, cwd: Option<&Path>) -> String {
    let path = cwd
        .and_then(|cwd| path.strip_prefix(cwd).ok())
        .unwrap_or(path);
    let path = path.strip_prefix(".").unwrap_or(path);
    path.display().to_string()
}

fn collect_files(root: &Path, exclude: &[String]) -> anyhow::Result<Vec<PathBuf>> {
    let base = if root.is_file() {
        root.parent().unwrap_or(root)
    } else {
        root
    };
    let mut overrides = OverrideBuilder::new(base);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_path_is_relative_to_cwd() {
        let cwd = Path::new("/repo");
        assert_eq!(
            display_path(Path::new("/repo/src/a.rs"), Some(cwd)),
            "src/a.rs"
        );
        assert_eq!(display_path(Path::new("./src/a.rs"), Some(cwd)), "src/a.rs");
        assert_eq!(
            display_path(Path::new("/other/a.rs"), Some(cwd)),
            "/other/a.rs"
        );
    }
}
