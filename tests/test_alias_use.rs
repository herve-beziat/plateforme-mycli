use mys3::commands::alias_use::Args;
use mys3::config::{AliasConfig, Config};
use mys3::error::MyS3Error;
use tempfile::tempdir;

fn sample_alias(url: &str) -> AliasConfig {
    AliasConfig {
        url: url.to_string(),
        access_key: "access".to_string(),
        secret_key: "secret".to_string(),
        region: "us-east-1".to_string(),
    }
}

#[test]
fn test_alias_use_switches_default() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");

    let mut config = Config::load_from(&config_file).unwrap();
    config.set_alias("local", sample_alias("http://localhost:9000"));
    config.set_alias("prod", sample_alias("https://s3.amazonaws.com"));
    config.save_to(&config_file).unwrap();

    assert_eq!(config.default, Some("local".to_string()));

    // Run set_default directly on loaded config
    config.set_default("prod").unwrap();
    config.save_to(&config_file).unwrap();

    let loaded = Config::load_from(&config_file).unwrap();
    assert_eq!(loaded.default, Some("prod".to_string()));
}

#[test]
fn test_alias_use_unknown_alias_returns_error() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");

    let mut config = Config::load_from(&config_file).unwrap();
    config.set_alias("local", sample_alias("http://localhost:9000"));
    config.save_to(&config_file).unwrap();

    let result = config.set_default("nonexistent");
    assert!(matches!(result, Err(MyS3Error::AliasNotFound(name)) if name == "nonexistent"));
}

#[test]
fn test_alias_use_args_struct() {
    let args = Args {
        alias_name: "prod".to_string(),
    };
    assert_eq!(args.alias_name, "prod");
}
