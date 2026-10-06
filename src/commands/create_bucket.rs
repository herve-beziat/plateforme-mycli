//! `mys3 create-bucket`: create a bucket.
//! Not implemented yet (issue #20).

use crate::error::MyS3Error;

/// Arguments of the `create-bucket` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the bucket to create
    pub bucket_name: String,

    /// Alias to use (default: the default alias)
    #[arg(long)]
    pub alias: Option<String>,

    /// Region of the bucket
    #[arg(long)]
    pub region: Option<String>,
}

/// Runs the `create-bucket` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("create-bucket"))
}
