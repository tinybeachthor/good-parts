pub mod cli;

use std::ffi::OsString;

use clap::Parser;

use crate::cli::Args;

/// Parse `args` (including the program name) and run the analysis.
pub fn run<I, T>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::parse_from(args);
    println!("analyzing {}", args.path.display());
    Ok(())
}
