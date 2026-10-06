//! `mys3 upload-file`: upload a local file to a bucket.
//! Not implemented yet (issue #23).

use std::path::PathBuf;

use crate::error::MyS3Error;

/// Arguments of the `upload-file` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Path of the local file to upload
    pub file_path: PathBuf,

    /// Name of the destination bucket
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Object key in the bucket (default: the file name)
    #[arg(long)]
    pub key: Option<String>,

    /// Replace the object if it already exists
    #[arg(long)]
    pub overwrite: bool,
}

/// Runs the `upload-file` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("upload-file"))
}
