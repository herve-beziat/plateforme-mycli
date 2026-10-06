//! `mys3 alias use`: switch the default alias.
//! Not implemented yet (issue #16).

use crate::error::MyS3Error;

/// Arguments of the `alias use` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the alias to use by default
    pub alias_name: String,
}

/// Runs the `alias use` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("alias use"))
}
