//! `mys3 alias remove`: delete an alias.

use crate::config::Config;
use crate::error::MyS3Error;
use crate::prompt;

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
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let name = args.alias_name;
    let mut config = Config::load()?;

    // An unknown alias is an error before any question is asked.
    if !config.aliases.contains_key(&name) {
        return Err(MyS3Error::AliasNotFound(name));
    }
    if !args.force && !prompt::confirm(&format!("Remove alias '{name}'?"))? {
        println!("Aborted.");
        return Ok(());
    }

    let was_default = config.default.as_deref() == Some(name.as_str());
    config.remove_alias(&name)?;
    config.save()?;

    println!("Alias '{name}' removed.");
    if was_default {
        println!("It was the default alias: run `mys3 alias use <name>` to choose another one.");
    }
    Ok(())
}
