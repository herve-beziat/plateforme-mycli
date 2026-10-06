//! `mys3 bucket-info`: show bucket details (name, creation date, object count, total size).
//! Not implemented yet (issue #22).

use crate::cli::OutputFormat;
use crate::error::MyS3Error;

/// Arguments of the `bucket-info` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Runs the `bucket-info` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("bucket-info"))
}
