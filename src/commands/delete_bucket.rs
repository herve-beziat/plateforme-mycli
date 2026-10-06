//! `mys3 delete-bucket`: delete a bucket.
//! Not implemented yet (issue #21).

use crate::error::MyS3Error;

/// Arguments of the `delete-bucket` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket to delete
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Skip the confirmation
    #[arg(short, long)]
    pub force: bool,

    /// Delete the objects of the bucket first (required for a non-empty bucket)
    #[arg(long)]
    pub recursive: bool,
}

/// Runs the `delete-bucket` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("delete-bucket"))
}
