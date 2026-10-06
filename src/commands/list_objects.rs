//! `mys3 list-objects`: list the objects of a bucket.
//! Not implemented yet (issue #24).

use crate::cli::OutputFormat;
use crate::error::MyS3Error;

/// Arguments of the `list-objects` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Only list the keys starting with this prefix
    #[arg(long)]
    pub prefix: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Runs the `list-objects` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("list-objects"))
}
