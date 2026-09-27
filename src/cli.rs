use std::path::PathBuf;

use clap::Parser;

use crate::output::Format;
use crate::report::SortKey;

const LONG_ABOUT: &str = "\
Find the complex parts of a code repository that are good refactoring candidates.

good-parts measures every function with arborist-metrics (cognitive complexity,
cyclomatic complexity and source lines of code) and ranks the functions, or
files, that are hardest to understand. Files ignored by .gitignore are skipped
inside git repositories.";

const AFTER_HELP: &str = "Run with --help for examples, scoring and exit codes.";

const AFTER_LONG_HELP: &str = "\
Examples:
  good-parts                                  Rank functions in the current directory
  good-parts src/parser.rs src/eval.rs        Check only specific files
  good-parts --files -n 10                    Top 10 files with the most complexity
  good-parts --format json                    Machine-readable output
  good-parts --fail-over-threshold --threshold 20
                                              Exit 1 if any function has cognitive > 20

Scoring:
  score = cognitive + 0.5 * cyclomatic + sloc / 20
  A file's score is the sum of its functions' scores.

Exit codes:
  0  Success
  1  --fail-over-threshold was given and a function exceeds --threshold
  2  Error (invalid arguments, unreadable path, ...)

Languages:
  Rust, Python, JavaScript, TypeScript, Java, Go. Other files are skipped.";

#[derive(Debug, Parser)]
#[command(
    name = "good-parts",
    version,
    about = "Find the complex parts of a code repository that are good refactoring candidates",
    long_about = LONG_ABOUT,
    after_help = AFTER_HELP,
    after_long_help = AFTER_LONG_HELP,
)]
pub struct Args {
    /// Files or directories to analyze.
    ///
    /// Directories are searched recursively. Passing specific files is a quick
    /// way to re-check code after editing it.
    #[arg(value_name = "PATH", default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Number of candidates to show.
    #[arg(short = 'n', long, default_value_t = 20)]
    pub top: usize,

    /// Metric to rank candidates by.
    #[arg(long, value_enum, default_value_t = SortKey::Score)]
    pub sort: SortKey,

    /// Cognitive complexity above which a function is flagged.
    ///
    /// Flagged functions are marked with `!` in the table and have
    /// `exceeds_threshold: true` in JSON. The default of 15 follows SonarSource.
    #[arg(long, default_value_t = 15)]
    pub threshold: u64,

    /// Hide functions with lower cognitive complexity than this.
    #[arg(long, default_value_t = 1)]
    pub min_cognitive: u64,

    /// Exit with code 1 if any function exceeds --threshold.
    ///
    /// Counts every analyzed function, not only the ones shown by --top.
    #[arg(long)]
    pub fail_over_threshold: bool,

    /// Rank files instead of functions.
    #[arg(long)]
    pub files: bool,

    /// Output format.
    ///
    /// `json` is a stable, versioned format intended for scripts and AI agents.
    #[arg(long, value_enum, default_value_t = Format::Table)]
    pub format: Format,

    /// Extra glob patterns to exclude, on top of .gitignore (repeatable).
    ///
    /// Patterns are relative to each PATH, e.g. `--exclude 'vendor/**'`.
    #[arg(long, value_name = "GLOB")]
    pub exclude: Vec<String>,

    /// Skip methods and only analyze free functions.
    #[arg(long)]
    pub no_methods: bool,
}
