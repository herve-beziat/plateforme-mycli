//! `mys3 alias set`: create or update an alias.

use crate::config::{AliasConfig, Config};
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

/// Validates that the URL starts with `http://` or `https://` and has a non-empty host.
pub fn validate_url(url: &str) -> Result<String, MyS3Error> {
    let trimmed = url.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(MyS3Error::InvalidUrl(url.to_string()));
    }

    let without_scheme = if let Some(rest) = trimmed.strip_prefix("http://") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("https://") {
        rest
    } else {
        return Err(MyS3Error::InvalidUrl(url.to_string()));
    };

    let host_and_port = without_scheme.split('/').next().unwrap_or("");
    if host_and_port.is_empty() {
        return Err(MyS3Error::InvalidUrl(url.to_string()));
    }

    // Strip trailing slashes from base URL
    Ok(trimmed.trim_end_matches('/').to_string())
}

/// Runs the `alias set` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let validated_url = validate_url(&args.url)?;

    let secret_key = match args.secret_key {
        Some(key) if !key.is_empty() => key,
        _ => {
            eprint!("Secret key: ");
            std::io::Write::flush(&mut std::io::stderr()).ok();
            rpassword::read_password()
                .map_err(|e| MyS3Error::ConfigRead("secret key input".to_string(), e))?
        }
    };

    let mut config = Config::load()?;

    let alias = AliasConfig {
        url: validated_url,
        access_key: args.access_key,
        secret_key,
        region: args.region.unwrap_or_else(|| "us-east-1".to_string()),
    };

    config.set_alias(&args.alias_name, alias);

    if args.default {
        config.set_default(&args.alias_name)?;
    }

    config.save()?;
    println!("Alias '{}' configured successfully.", args.alias_name);
    Ok(())
}
