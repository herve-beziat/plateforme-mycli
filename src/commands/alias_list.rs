//! `mys3 alias list`: list the aliases (keys are never displayed).

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::MyS3Error;

/// Message shown when no alias is configured.
pub const NO_ALIAS_MESSAGE: &str = "No aliases configured. Run `mys3 alias set` to add one.";

/// Arguments of the `alias list` command.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, ignore_case = true)]
    pub output: OutputFormat,
}

/// What `alias list` shows about an alias. There is no field for the keys, so
/// they can never be displayed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AliasItem {
    pub name: String,
    pub url: String,
    pub region: String,
    pub default: bool,
}

/// The aliases of `config`, sorted by name.
pub fn alias_items(config: &Config) -> Vec<AliasItem> {
    config
        .aliases
        .iter()
        .map(|(name, alias)| AliasItem {
            name: name.clone(),
            url: alias.url.clone(),
            region: alias.region.clone(),
            default: config.default.as_deref() == Some(name.as_str()),
        })
        .collect()
}

/// The aliases as a table with aligned columns; `*` marks the default alias.
pub fn format_table(items: &[AliasItem]) -> String {
    let name_width = items
        .iter()
        .map(|item| item.name.len())
        .chain(["NAME".len()])
        .max()
        .unwrap_or_default();
    let url_width = items
        .iter()
        .map(|item| item.url.len())
        .chain(["URL".len()])
        .max()
        .unwrap_or_default();

    let mut table = format!("  {:name_width$}  {:url_width$}  REGION\n", "NAME", "URL");
    for item in items {
        let marker = if item.default { '*' } else { ' ' };
        table.push_str(&format!(
            "{marker} {:name_width$}  {:url_width$}  {}\n",
            item.name, item.url, item.region
        ));
    }
    table
}

/// Runs the `alias list` command.
pub fn run(args: Args) -> Result<(), MyS3Error> {
    let items = alias_items(&Config::load()?);

    match args.output {
        OutputFormat::Text if items.is_empty() => println!("{NO_ALIAS_MESSAGE}"),
        OutputFormat::Text => print!("{}", format_table(&items)),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&items).expect("the aliases are always serializable")
        ),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AliasConfig;

    fn config() -> Config {
        let mut config = Config::default();
        config.set_alias(
            "prod",
            AliasConfig {
                url: "https://s3.example.com".to_string(),
                access_key: "prod-access".to_string(),
                secret_key: "prod-secret".to_string(),
                region: "eu-west-1".to_string(),
            },
        );
        config.set_alias(
            "local",
            AliasConfig {
                url: "http://localhost:9000".to_string(),
                access_key: "local-access".to_string(),
                secret_key: "local-secret".to_string(),
                region: "us-east-1".to_string(),
            },
        );
        config
    }

    #[test]
    fn items_are_sorted_and_mark_the_default_alias() {
        let items = alias_items(&config());
        let names: Vec<_> = items.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, ["local", "prod"]);
        assert!(!items[0].default);
        assert!(items[1].default, "the first alias set becomes the default");
    }

    #[test]
    fn table_has_aligned_columns_and_a_default_marker() {
        let table = format_table(&alias_items(&config()));
        assert_eq!(
            table,
            "  NAME   URL                     REGION\n\
             \x20 local  http://localhost:9000   us-east-1\n\
             * prod   https://s3.example.com  eu-west-1\n"
        );
    }

    #[test]
    fn json_has_no_keys() {
        let json = serde_json::to_string(&alias_items(&config())).unwrap();
        assert!(!json.contains("access"));
        assert!(!json.contains("secret"));
    }
}
