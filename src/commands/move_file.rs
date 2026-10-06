//! `mys3 move-file`: move an object between buckets of the same server (copy, then delete the source).
//! Not implemented yet (issue #29).

use crate::error::MyS3Error;

/// Arguments of the `move-file` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the source bucket
    pub source_bucket: String,

    /// Key of the object in the source bucket
    pub object_key: String,

    /// Name of the destination bucket
    pub destination_bucket: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Key in the destination bucket (default: same key)
    #[arg(long, value_name = "NEW_KEY")]
    pub key: Option<String>,

    /// Replace the object if it already exists at the destination
    #[arg(long)]
    pub overwrite: bool,
}

/// Runs the `move-file` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("move-file"))
}
