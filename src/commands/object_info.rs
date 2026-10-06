//! `mys3 object-info`: show object details (name, size, last modified, content type, ETag).
//! Not implemented yet (issue #27).

use crate::cli::OutputFormat;
use crate::error::MyS3Error;

/// Arguments of the `object-info` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Key of the object
    pub object_key: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// Runs the `object-info` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("object-info"))
}
