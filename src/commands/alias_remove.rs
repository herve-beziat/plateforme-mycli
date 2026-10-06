//! `mys3 alias remove`: delete an alias.
//! Not implemented yet (issue #18).

use crate::error::MyS3Error;

/// Arguments of the `alias remove` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the alias to delete
    pub alias_name: String,

    /// Skip the confirmation
    #[arg(short, long)]
    pub force: bool,
}

/// Runs the `alias remove` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("alias remove"))
}
