//! Configuration file (`~/.mys3/config.json`): the saved aliases, the default
//! alias, and the resolution of the alias and keys used by a command.

use std::collections::BTreeMap;
use std::fs::{self, DirBuilder, OpenOptions, Permissions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::MyS3Error;

const CONFIG_DIR: &str = ".mys3";
const CONFIG_FILE: &str = "config.json";
const DEFAULT_REGION: &str = "us-east-1";

fn default_region() -> String {
    DEFAULT_REGION.to_string()
}

/// A server saved under a name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AliasConfig {
    /// Base URL of the server (e.g. `http://localhost:9000`).
    pub url: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default = "default_region")]
    pub region: String,
}

/// Content of the configuration file.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Alias used when no `--alias` is given.
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub aliases: BTreeMap<String, AliasConfig>,
}

/// Access key and secret key given by one source (an option or the environment).
#[derive(Debug, Default, Clone)]
pub struct Credentials {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

impl Credentials {
    /// Reads `MYS3_ACCESS_KEY` and `MYS3_SECRET_KEY`.
    pub fn from_env() -> Self {
        Self {
            access_key: std::env::var("MYS3_ACCESS_KEY").ok(),
            secret_key: std::env::var("MYS3_SECRET_KEY").ok(),
        }
    }
}

/// Everything a command needs to call the server.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedAlias {
    pub name: String,
    pub url: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

impl Config {
    /// Path of the configuration file: `~/.mys3/config.json`.
    pub fn path() -> Result<PathBuf, MyS3Error> {
        let home = dirs::home_dir().ok_or(MyS3Error::HomeNotFound)?;
        Ok(home.join(CONFIG_DIR).join(CONFIG_FILE))
    }

    /// Loads `~/.mys3/config.json`.
    pub fn load() -> Result<Self, MyS3Error> {
        Self::load_from(&Self::path()?)
    }

    /// Saves to `~/.mys3/config.json`.
    pub fn save(&self) -> Result<(), MyS3Error> {
        self.save_to(&Self::path()?)
    }

    /// Loads the configuration at `path`. A missing file is an empty configuration.
    pub fn load_from(path: &Path) -> Result<Self, MyS3Error> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => return Err(MyS3Error::ConfigRead(path.display().to_string(), err)),
        };
        serde_json::from_str(&content)
            .map_err(|err| MyS3Error::InvalidConfig(path.display().to_string(), err))
    }

    /// Saves the configuration at `path`, readable and writable by the owner only (`600`).
    pub fn save_to(&self, path: &Path) -> Result<(), MyS3Error> {
        let write_error = |err| MyS3Error::ConfigWrite(path.display().to_string(), err);

        if let Some(dir) = path.parent() {
            DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(dir)
                .map_err(write_error)?;
        }

        let mut content =
            serde_json::to_string_pretty(self).expect("the configuration is always serializable");
        content.push('\n');

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(write_error)?;
        // `mode` only applies when the file is created: fix a file that already
        // existed with wider rights.
        file.set_permissions(Permissions::from_mode(0o600))
            .map_err(write_error)?;
        file.write_all(content.as_bytes()).map_err(write_error)
    }

    /// Adds or replaces an alias. The first alias becomes the default one.
    pub fn set_alias(&mut self, name: &str, alias: AliasConfig) {
        self.aliases.insert(name.to_string(), alias);
        if self.default.is_none() {
            self.default = Some(name.to_string());
        }
    }

    /// Makes `name` the default alias.
    pub fn set_default(&mut self, name: &str) -> Result<(), MyS3Error> {
        if !self.aliases.contains_key(name) {
            return Err(MyS3Error::AliasNotFound(name.to_string()));
        }
        self.default = Some(name.to_string());
        Ok(())
    }

    /// Deletes an alias, and the default if it pointed to it.
    pub fn remove_alias(&mut self, name: &str) -> Result<(), MyS3Error> {
        if self.aliases.remove(name).is_none() {
            return Err(MyS3Error::AliasNotFound(name.to_string()));
        }
        if self.default.as_deref() == Some(name) {
            self.default = None;
        }
        Ok(())
    }

    /// Picks the alias (`alias` given with `--alias`, otherwise the default one)
    /// and its keys, each one taken from `option`, then `env`, then the file.
    ///
    /// The environment is passed in rather than read here, so tests do not
    /// depend on the process environment.
    pub fn resolve(
        &self,
        alias: Option<&str>,
        option: Credentials,
        env: Credentials,
    ) -> Result<ResolvedAlias, MyS3Error> {
        let name = alias
            .or(self.default.as_deref())
            .ok_or(MyS3Error::NoDefaultAlias)?;
        let saved = self
            .aliases
            .get(name)
            .ok_or_else(|| MyS3Error::AliasNotFound(name.to_string()))?;

        Ok(ResolvedAlias {
            name: name.to_string(),
            url: saved.url.clone(),
            access_key: option
                .access_key
                .or(env.access_key)
                .unwrap_or_else(|| saved.access_key.clone()),
            secret_key: option
                .secret_key
                .or(env.secret_key)
                .unwrap_or_else(|| saved.secret_key.clone()),
            region: saved.region.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alias(access_key: &str, secret_key: &str) -> AliasConfig {
        AliasConfig {
            url: "http://localhost:9000".to_string(),
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
            region: DEFAULT_REGION.to_string(),
        }
    }

    /// Two aliases: `local` (the default) and `prod`.
    fn config() -> Config {
        let mut config = Config::default();
        config.set_alias("local", alias("file-access", "file-secret"));
        config.set_alias("prod", alias("prod-access", "prod-secret"));
        config
    }

    fn credentials(access_key: Option<&str>, secret_key: Option<&str>) -> Credentials {
        Credentials {
            access_key: access_key.map(str::to_string),
            secret_key: secret_key.map(str::to_string),
        }
    }

    fn mode(path: &Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn missing_file_is_an_empty_config() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load_from(&dir.path().join(CONFIG_FILE)).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn save_then_load_gives_the_same_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE);
        let config = config();
        config.save_to(&path).unwrap();
        assert_eq!(Config::load_from(&path).unwrap(), config);
    }

    #[test]
    fn save_creates_a_700_directory_and_a_600_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_DIR).join(CONFIG_FILE);
        config().save_to(&path).unwrap();
        assert_eq!(mode(&path), 0o600);
        assert_eq!(mode(path.parent().unwrap()), 0o700);
    }

    #[test]
    fn save_restricts_an_existing_file_to_600() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE);
        fs::write(&path, "{}").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o644)).unwrap();
        config().save_to(&path).unwrap();
        assert_eq!(mode(&path), 0o600);
    }

    #[test]
    fn invalid_json_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE);
        fs::write(&path, "not json").unwrap();
        assert!(matches!(
            Config::load_from(&path),
            Err(MyS3Error::InvalidConfig(..))
        ));
    }

    #[test]
    fn region_defaults_to_us_east_1() {
        let config: Config = serde_json::from_str(
            r#"{"aliases": {"local": {"url": "http://localhost:9000", "access_key": "a", "secret_key": "s"}}}"#,
        )
        .unwrap();
        assert_eq!(config.aliases["local"].region, DEFAULT_REGION);
    }

    #[test]
    fn first_alias_becomes_the_default() {
        assert_eq!(config().default.as_deref(), Some("local"));
    }

    #[test]
    fn alias_option_wins_over_the_default() {
        let resolved = config()
            .resolve(Some("prod"), Credentials::default(), Credentials::default())
            .unwrap();
        assert_eq!(resolved.name, "prod");
        assert_eq!(resolved.access_key, "prod-access");
    }

    #[test]
    fn default_alias_is_used_without_option() {
        let resolved = config()
            .resolve(None, Credentials::default(), Credentials::default())
            .unwrap();
        assert_eq!(resolved.name, "local");
    }

    #[test]
    fn unknown_alias_is_an_error() {
        let result = config().resolve(Some("nope"), Credentials::default(), Credentials::default());
        assert!(matches!(result, Err(MyS3Error::AliasNotFound(name)) if name == "nope"));
    }

    #[test]
    fn no_default_alias_is_an_error() {
        let result =
            Config::default().resolve(None, Credentials::default(), Credentials::default());
        assert!(matches!(result, Err(MyS3Error::NoDefaultAlias)));
    }

    #[test]
    fn keys_come_from_the_file_by_default() {
        let resolved = config()
            .resolve(None, Credentials::default(), Credentials::default())
            .unwrap();
        assert_eq!(resolved.access_key, "file-access");
        assert_eq!(resolved.secret_key, "file-secret");
    }

    #[test]
    fn env_wins_over_the_file() {
        let env = credentials(Some("env-access"), Some("env-secret"));
        let resolved = config().resolve(None, Credentials::default(), env).unwrap();
        assert_eq!(resolved.access_key, "env-access");
        assert_eq!(resolved.secret_key, "env-secret");
    }

    #[test]
    fn option_wins_over_env_and_file() {
        let option = credentials(Some("opt-access"), Some("opt-secret"));
        let env = credentials(Some("env-access"), Some("env-secret"));
        let resolved = config().resolve(None, option, env).unwrap();
        assert_eq!(resolved.access_key, "opt-access");
        assert_eq!(resolved.secret_key, "opt-secret");
    }

    #[test]
    fn priority_applies_to_each_key_separately() {
        let option = credentials(None, Some("opt-secret"));
        let env = credentials(Some("env-access"), None);
        let resolved = config().resolve(None, option, env).unwrap();
        assert_eq!(resolved.access_key, "env-access");
        assert_eq!(resolved.secret_key, "opt-secret");
    }
}
