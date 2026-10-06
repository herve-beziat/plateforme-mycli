//! `mys3 download-file`: download an object to the local disk.
//! Not implemented yet (issue #25).

use std::path::PathBuf;

use crate::error::MyS3Error;

/// Arguments of the `download-file` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Key of the object to download
    pub object_key: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Local destination path (default: current directory, object name)
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Replace the local file if it already exists
    #[arg(long)]
    pub overwrite: bool,
}

/// Runs the `download-file` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("download-file"))
}
