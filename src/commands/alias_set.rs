//! `mys3 alias set`: create or update an alias.
//! Not implemented yet (issue #15).

use crate::error::MyS3Error;

/// Arguments of the `alias set` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the alias
    pub alias_name: String,

    /// URL of the server (e.g. http://localhost:9000)
    pub url: String,

    /// Access key
    pub access_key: String,

    /// Secret key (prompted with hidden input when omitted)
    pub secret_key: Option<String>,

    /// Make this alias the default one
    #[arg(long)]
    pub default: bool,

    /// Region of the server
    #[arg(long)]
    pub region: Option<String>,
}

/// Runs the `alias set` command.
pub fn run(_args: Args) -> Result<(), MyS3Error> {
    Err(MyS3Error::NotImplemented("alias set"))
}
