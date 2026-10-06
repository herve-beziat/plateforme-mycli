//! `mys3 sync`: one-way sync from a local folder to a bucket.
//! Not implemented yet (issue #30).

use std::path::PathBuf;

use crate::error::MyS3Error;

/// Arguments of the `sync` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Local folder whose contents are synchronised
    pub local_folder: PathBuf,

    /// Name of the destination bucket
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Destination prefix in the bucket
    #[arg(long)]
    pub prefix: Option<String>,

    /// Show what would be done without doing it
    #[arg(long)]
    pub dry_run: bool,

    /// Also remove from the bucket the objects that no longer exist locally
    #[arg(long)]
    pub delete: bool,

    /// Skip the confirmation
    #[arg(short, long)]
    pub force: bool,
}

/// Runs the `sync` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("sync"))
}
