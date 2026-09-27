use std::path::PathBuf;

use clap::Parser;

use crate::output::Format;
use crate::report::SortKey;

/// Find the complex parts of a code repository that are good refactoring candidates.
#[derive(Debug, Parser)]
#[command(name = "good-parts", version, about)]
pub struct Args {
    /// Files or directories to analyze.
    #[arg(value_name = "PATH", default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Number of candidates to show.
    #[arg(short = 'n', long, default_value_t = 20)]
    pub top: usize,

    /// Metric to rank candidates by.
    #[arg(long, value_enum, default_value_t = SortKey::Score)]
    pub sort: SortKey,

    /// Cognitive complexity above which a function is flagged.
    #[arg(long, default_value_t = 15)]
    pub threshold: u64,

    /// Hide functions with lower cognitive complexity than this.
    #[arg(long, default_value_t = 1)]
    pub min_cognitive: u64,

    /// Exit with code 1 if any function exceeds --threshold.
    #[arg(long)]
    pub fail_over_threshold: bool,

    /// Rank files instead of functions.
    #[arg(long)]
    pub files: bool,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Table)]
    pub format: Format,

    /// Extra glob patterns to exclude, on top of .gitignore (repeatable).
    #[arg(long, value_name = "GLOB")]
    pub exclude: Vec<String>,

    /// Skip methods and only analyze free functions.
    #[arg(long)]
    pub no_methods: bool,
}
