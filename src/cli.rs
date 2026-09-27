use std::path::PathBuf;

use clap::Parser;

/// Find the complex parts of a code repository that are good refactoring candidates.
#[derive(Debug, Parser)]
#[command(name = "good-parts", version, about)]
pub struct Args {
    /// Repository root to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Cognitive complexity above which a function is flagged.
    #[arg(long, default_value_t = 15)]
    pub threshold: u64,

    /// Extra glob patterns to exclude, on top of .gitignore (repeatable).
    #[arg(long, value_name = "GLOB")]
    pub exclude: Vec<String>,

    /// Skip methods and only analyze free functions.
    #[arg(long)]
    pub no_methods: bool,
}
