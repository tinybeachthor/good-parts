pub mod cli;
pub mod output;
pub mod report;
pub mod scan;

use std::ffi::OsString;
use std::process::ExitCode;

use arborist::AnalysisConfig;
use clap::Parser;

use crate::cli::Args;
use crate::output::{Format, Summary};

/// Exit code when `--fail-over-threshold` finds functions over the threshold.
pub const EXIT_OVER_THRESHOLD: u8 = 1;
/// Exit code for runtime errors (clap also uses 2 for usage errors).
pub const EXIT_ERROR: u8 = 2;

/// Entry point shared by both binaries: runs and maps errors to [`EXIT_ERROR`].
pub fn main_with<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    run(args).unwrap_or_else(|err| {
        eprintln!("error: {err:#}");
        ExitCode::from(EXIT_ERROR)
    })
}

/// Parse `args` (including the program name) and run the analysis.
pub fn run<I, T>(args: I) -> anyhow::Result<ExitCode>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::parse_from(args);
    let config = AnalysisConfig {
        cognitive_threshold: Some(args.threshold),
        include_methods: !args.no_methods,
    };

    let reports = scan::scan(&args.paths, &args.exclude, &config)?;
    let summary = Summary::new(&reports, args.threshold);
    let mut out = std::io::stdout().lock();

    if args.files {
        let hotspots = report::file_hotspots(&reports, args.sort, args.top);
        match args.format {
            Format::Table => output::write_file_table(&mut out, &summary, &hotspots)?,
            Format::Json => output::write_json(&mut out, "files", &summary, &hotspots)?,
        }
    } else {
        let hotspots = report::function_hotspots(&reports, args.min_cognitive, args.sort, args.top);
        match args.format {
            Format::Table => output::write_function_table(&mut out, &summary, &hotspots)?,
            Format::Json => output::write_json(&mut out, "functions", &summary, &hotspots)?,
        }
    }

    if args.fail_over_threshold && summary.over_threshold > 0 {
        return Ok(ExitCode::from(EXIT_OVER_THRESHOLD));
    }
    Ok(ExitCode::SUCCESS)
}
