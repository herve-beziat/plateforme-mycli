//! `mys3 alias use`: switch the default alias.

use crate::config::Config;
use crate::error::MyS3Error;

/// Arguments of the `alias use` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Name of the alias to use by default
    pub alias_name: String,
}

/// Runs the `alias use` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let mut config = Config::load()?;
    config.set_default(&args.alias_name)?;
    config.save()?;
    println!("Default alias set to '{}'.", args.alias_name);
    Ok(())
}
