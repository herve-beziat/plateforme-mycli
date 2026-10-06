//! `mys3 delete-file`: delete an object.
//! Not implemented yet (issue #26).

use crate::error::MyS3Error;

/// Arguments of the `delete-file` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket
    pub bucket_name: String,

    /// Key of the object to delete
    pub object_key: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Skip the confirmation
    #[arg(short, long)]
    pub force: bool,
}

/// Runs the `delete-file` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("delete-file"))
}
