use arborist::{FileReport, FunctionMetrics, Language};
use clap::ValueEnum;

/// Metric used to rank hotspots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortKey {
    Score,
    Cognitive,
    Cyclomatic,
    Sloc,
}

/// A function that is a candidate for refactoring.
#[derive(Debug, Clone)]
pub struct FunctionHotspot {
    pub file: String,
    pub language: Language,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub cognitive: u64,
    pub cyclomatic: u64,
    pub sloc: u64,
    pub score: f64,
    pub exceeds_threshold: bool,
}

/// A file that concentrates complex functions.
#[derive(Debug, Clone)]
pub struct FileHotspot {
    pub file: String,
    pub language: Language,
    pub functions: usize,
    pub over_threshold: usize,
    pub total_cognitive: u64,
    pub max_cognitive: u64,
    pub cyclomatic: u64,
    pub sloc: u64,
    pub score: f64,
}

/// Refactoring priority of a single function.
///
/// Cognitive complexity dominates because it best tracks how hard code is to
/// understand; cyclomatic complexity and length act as tie-breakers.
pub fn function_score(cognitive: u64, cyclomatic: u64, sloc: u64) -> f64 {
    cognitive as f64 + cyclomatic as f64 * 0.5 + sloc as f64 / 20.0
}

fn score_of(f: &FunctionMetrics) -> f64 {
    function_score(f.cognitive, f.cyclomatic, f.sloc)
}

/// All functions with at least `min_cognitive`, ranked by `sort`, best candidates first.
pub fn function_hotspots(
    reports: &[FileReport],
    min_cognitive: u64,
    sort: SortKey,
    top: usize,
) -> Vec<FunctionHotspot> {
    let mut hotspots: Vec<FunctionHotspot> = reports
        .iter()
        .flat_map(|report| {
            report
                .functions
                .iter()
                .filter(|f| f.cognitive >= min_cognitive)
                .map(|f| FunctionHotspot {
                    file: report.path.clone(),
                    language: report.language,
                    name: f.name.clone(),
                    start_line: f.start_line,
                    end_line: f.end_line,
                    cognitive: f.cognitive,
                    cyclomatic: f.cyclomatic,
                    sloc: f.sloc,
                    score: score_of(f),
                    exceeds_threshold: f.exceeds_threshold == Some(true),
                })
        })
        .collect();

    hotspots.sort_by(|a, b| {
        let key = |h: &FunctionHotspot| match sort {
            SortKey::Score => h.score,
            SortKey::Cognitive => h.cognitive as f64,
            SortKey::Cyclomatic => h.cyclomatic as f64,
            SortKey::Sloc => h.sloc as f64,
        };
        key(b).total_cmp(&key(a)).then(b.score.total_cmp(&a.score))
    });
    hotspots.truncate(top);
    hotspots
}

/// Files ranked by `sort`, best candidates first. Files without functions are skipped.
pub fn file_hotspots(reports: &[FileReport], sort: SortKey, top: usize) -> Vec<FileHotspot> {
    let mut hotspots: Vec<FileHotspot> = reports
        .iter()
        .filter(|report| !report.functions.is_empty())
        .map(|report| FileHotspot {
            file: report.path.clone(),
            language: report.language,
            functions: report.functions.len(),
            over_threshold: report
                .functions
                .iter()
                .filter(|f| f.exceeds_threshold == Some(true))
                .count(),
            total_cognitive: report.file_cognitive,
            max_cognitive: report
                .functions
                .iter()
                .map(|f| f.cognitive)
                .max()
                .unwrap_or(0),
            cyclomatic: report.file_cyclomatic,
            sloc: report.file_sloc,
            score: report.functions.iter().map(score_of).sum(),
        })
        .collect();

    hotspots.sort_by(|a, b| {
        let key = |h: &FileHotspot| match sort {
            SortKey::Score => h.score,
            SortKey::Cognitive => h.total_cognitive as f64,
            SortKey::Cyclomatic => h.cyclomatic as f64,
            SortKey::Sloc => h.sloc as f64,
        };
        key(b).total_cmp(&key(a)).then(b.score.total_cmp(&a.score))
    });
    hotspots.truncate(top);
    hotspots
}

#[cfg(test)]
mod tests {
    use super::*;

    fn func(name: &str, cognitive: u64, cyclomatic: u64, sloc: u64) -> FunctionMetrics {
        FunctionMetrics {
            name: name.to_string(),
            start_line: 1,
            end_line: 1 + sloc as usize,
            cognitive,
            cyclomatic,
            sloc,
            exceeds_threshold: Some(cognitive > 15),
        }
    }

    fn file(path: &str, functions: Vec<FunctionMetrics>) -> FileReport {
        FileReport {
            path: path.to_string(),
            language: Language::Rust,
            file_cognitive: functions.iter().map(|f| f.cognitive).sum(),
            file_cyclomatic: functions.iter().map(|f| f.cyclomatic).sum(),
            file_sloc: functions.iter().map(|f| f.sloc).sum(),
            functions,
        }
    }

    fn sample() -> Vec<FileReport> {
        vec![
            file(
                "a.rs",
                vec![func("simple", 1, 1, 5), func("long", 2, 2, 400)],
            ),
            file(
                "b.rs",
                vec![func("tangled", 20, 10, 60), func("trivial", 0, 1, 2)],
            ),
            file("empty.rs", vec![]),
        ]
    }

    #[test]
    fn score_weights_cognitive_highest() {
        assert_eq!(function_score(10, 4, 40), 14.0);
        assert!(function_score(5, 0, 0) > function_score(0, 5, 0));
    }

    #[test]
    fn functions_ranked_by_score() {
        let names: Vec<_> = function_hotspots(&sample(), 1, SortKey::Score, 10)
            .into_iter()
            .map(|h| h.name)
            .collect();
        assert_eq!(names, ["tangled", "long", "simple"]);
    }

    #[test]
    fn functions_filtered_sorted_and_truncated() {
        let hotspots = function_hotspots(&sample(), 0, SortKey::Sloc, 2);
        let names: Vec<_> = hotspots.iter().map(|h| h.name.as_str()).collect();
        assert_eq!(names, ["long", "tangled"]);
        assert!(hotspots[1].exceeds_threshold);
    }

    #[test]
    fn files_ranked_and_empty_skipped() {
        let hotspots = file_hotspots(&sample(), SortKey::Score, 10);
        let files: Vec<_> = hotspots.iter().map(|h| h.file.as_str()).collect();
        assert_eq!(files, ["b.rs", "a.rs"]);
        assert_eq!(hotspots[0].over_threshold, 1);
        assert_eq!(hotspots[0].max_cognitive, 20);
    }
}
