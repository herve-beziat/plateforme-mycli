use mys3::commands::alias_set::validate_url;
use mys3::config::{AliasConfig, Config};
use mys3::error::MyS3Error;
use tempfile::tempdir;

#[test]
fn test_validate_url_valid() {
    assert_eq!(
        validate_url("http://localhost:9000").unwrap(),
        "http://localhost:9000"
    );
    assert_eq!(
        validate_url("http://localhost:9000/").unwrap(),
        "http://localhost:9000"
    );
    assert_eq!(
        validate_url("https://s3.amazonaws.com").unwrap(),
        "https://s3.amazonaws.com"
    );
    assert_eq!(
        validate_url("https://s3.amazonaws.com/").unwrap(),
        "https://s3.amazonaws.com"
    );
}

#[test]
fn test_validate_url_invalid() {
    assert!(matches!(
        validate_url("ftp://localhost"),
        Err(MyS3Error::InvalidUrl(_))
    ));
    assert!(matches!(
        validate_url("localhost:9000"),
        Err(MyS3Error::InvalidUrl(_))
    ));
    assert!(matches!(
        validate_url("http://"),
        Err(MyS3Error::InvalidUrl(_))
    ));
    assert!(matches!(
        validate_url("https:///"),
        Err(MyS3Error::InvalidUrl(_))
    ));
}

#[test]
fn test_alias_set_saves_to_config() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");

    let mut config = Config::load_from(&config_file).unwrap();
    assert_eq!(config.aliases.len(), 0);

    let alias = AliasConfig {
        url: validate_url("http://localhost:9000").unwrap(),
        access_key: "minioadmin".to_string(),
        secret_key: "minioadmin".to_string(),
        region: "us-east-1".to_string(),
    };

    config.set_alias("local", alias.clone());
    config.save_to(&config_file).unwrap();

    let loaded = Config::load_from(&config_file).unwrap();
    assert_eq!(loaded.aliases.get("local"), Some(&alias));
    assert_eq!(loaded.default, Some("local".to_string()));
}

#[test]
fn test_alias_set_updates_existing_alias() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");

    let mut config = Config::load_from(&config_file).unwrap();
    let alias1 = AliasConfig {
        url: "http://localhost:9000".to_string(),
        access_key: "key1".to_string(),
        secret_key: "secret1".to_string(),
        region: "us-east-1".to_string(),
    };
    config.set_alias("local", alias1);

    let alias2 = AliasConfig {
        url: "http://localhost:9000".to_string(),
        access_key: "key2".to_string(),
        secret_key: "secret2".to_string(),
        region: "eu-west-1".to_string(),
    };
    config.set_alias("local", alias2.clone());
    config.save_to(&config_file).unwrap();

    let loaded = Config::load_from(&config_file).unwrap();
    assert_eq!(loaded.aliases.get("local"), Some(&alias2));
}

#[test]
fn test_alias_set_explicit_default() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");

    let mut config = Config::load_from(&config_file).unwrap();
    let alias1 = AliasConfig {
        url: "http://localhost:9000".to_string(),
        access_key: "key1".to_string(),
        secret_key: "secret1".to_string(),
        region: "us-east-1".to_string(),
    };
    config.set_alias("local", alias1);

    let alias2 = AliasConfig {
        url: "http://remote:9000".to_string(),
        access_key: "key2".to_string(),
        secret_key: "secret2".to_string(),
        region: "us-east-1".to_string(),
    };
    config.set_alias("remote", alias2);
    config.set_default("remote").unwrap();
    config.save_to(&config_file).unwrap();

    let loaded = Config::load_from(&config_file).unwrap();
    assert_eq!(loaded.default, Some("remote".to_string()));
}
