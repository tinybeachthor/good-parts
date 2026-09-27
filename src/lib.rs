pub mod cli;
pub mod report;
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
    if args.files {
        for h in report::file_hotspots(&reports, args.sort, args.top) {
            println!(
                "{} score={:.1} cognitive={}",
                h.file, h.score, h.total_cognitive
            );
        }
    } else {
        for h in report::function_hotspots(&reports, args.min_cognitive, args.sort, args.top) {
            println!(
                "{}:{} {} score={:.1}",
                h.file, h.start_line, h.name, h.score
            );
        }
    }
    Ok(())
}
