pub mod cli;
pub mod scan;

use std::ffi::OsString;

use arborist::AnalysisConfig;
use clap::Parser;

use crate::cli::Args;

/// Parse `args` (including the program name) and run the analysis.
pub fn run<I, T>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::parse_from(args);
    let config = AnalysisConfig {
        cognitive_threshold: Some(args.threshold),
        include_methods: !args.no_methods,
    };

    let reports = scan::scan(&args.path, &args.exclude, &config)?;
    for report in &reports {
        println!(
            "{} ({}): {} functions, cognitive={}, sloc={}",
            report.path,
            report.language,
            report.functions.len(),
            report.file_cognitive,
            report.file_sloc,
        );
    }
    Ok(())
}
