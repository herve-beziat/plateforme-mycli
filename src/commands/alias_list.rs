//! `mys3 alias list`: list the aliases (keys are never displayed).
//! Not implemented yet (issue #17).

use crate::cli::OutputFormat;
use crate::error::MyS3Error;

/// Arguments of the `alias list` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Runs the `alias list` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("alias list"))
}
