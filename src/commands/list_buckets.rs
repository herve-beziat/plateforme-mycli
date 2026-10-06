//! `mys3 list-buckets`: list all buckets on the server.
//! Not implemented yet (issue #19).

use crate::cli::OutputFormat;
use crate::error::MyS3Error;

/// Arguments of the `list-buckets` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Runs the `list-buckets` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("list-buckets"))
}
