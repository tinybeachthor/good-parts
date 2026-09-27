use std::path::PathBuf;

use clap::Parser;

/// Find the complex parts of a code repository that are good refactoring candidates.
#[derive(Debug, Parser)]
#[command(name = "good-parts", version, about)]
pub struct Args {
    /// Repository root to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}
