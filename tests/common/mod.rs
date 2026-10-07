//! Shared helpers of the functional tests.
//!
//! A functional test runs the real `mys3` binary and checks its output and its
//! exit code. Two helpers keep these tests isolated and repeatable:
//!
//! - [`TestEnv`] gives the binary a temporary home directory, so a test never
//!   reads or writes the real `~/.mys3/config.json`.
//! - [`TempBucket`] is a bucket with a unique name on the test server, deleted
//!   with its objects when the test ends, even if the test fails.
//!
//! Tests that talk to the server need MinIO (`docker compose up -d`) and its
//! keys in `MYS3_ACCESS_KEY` and `MYS3_SECRET_KEY`. Mark them `#[ignore]` and
//! run them with `cargo test -- --ignored`.
//!
//! Example (in `tests/test_<command>.rs`):
//!
//! ```ignore
//! mod common;
//!
//! use common::TestEnv;
//!
//! #[test]
//! #[ignore = "needs MinIO"]
//! fn upload_file_stores_the_object() {
//!     let env = TestEnv::with_server();
//!     let bucket = env.new_bucket();
//!
//!     env.cmd()
//!         .args(["upload-file", "Cargo.toml", bucket.name()])
//!         .assert()
//!         .success();
//!
//!     assert_eq!(bucket.object_keys(), ["Cargo.toml"]);
//! }
//! ```

// Each test file is compiled on its own and uses only some of the helpers.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd::Command;
use mys3::client::{Response, S3Client};
use mys3::config::{AliasConfig, Config, ResolvedAlias};
use tempfile::TempDir;

/// Name of the default alias configured by [`TestEnv::with_server`].
pub const TEST_ALIAS: &str = "test";

/// URL of the test server, unless `MYS3_TEST_URL` is set.
const DEFAULT_SERVER_URL: &str = "http://localhost:9000";

/// A temporary home directory for the `mys3` binary, deleted at the end of the test.
pub struct TestEnv {
    home: TempDir,
    server: Option<ResolvedAlias>,
}

impl TestEnv {
    /// An empty home directory: no configuration file, no alias.
    pub fn new() -> Self {
        Self {
            home: tempfile::tempdir().expect("cannot create a temporary home directory"),
            server: None,
        }
    }

    /// A home directory whose default alias (`test`) points to the test server.
    ///
    /// The keys are read from `MYS3_ACCESS_KEY` and `MYS3_SECRET_KEY`.
    pub fn with_server() -> Self {
        let mut env = Self::new();
        let server = server_alias();

        let mut config = Config::default();
        config.set_alias(
            TEST_ALIAS,
            AliasConfig {
                url: server.url.clone(),
                access_key: server.access_key.clone(),
                secret_key: server.secret_key.clone(),
                region: server.region.clone(),
            },
        );
        config
            .save_to(&env.config_path())
            .expect("cannot write the test configuration");

        env.server = Some(server);
        env
    }

    /// The `mys3` binary, ready to receive its arguments, using this home directory.
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("mys3"));
        cmd.env("HOME", self.home.path());
        cmd
    }

    /// The temporary home directory.
    pub fn home(&self) -> &Path {
        self.home.path()
    }

    /// Where the binary reads and writes its configuration in this environment.
    pub fn config_path(&self) -> PathBuf {
        self.home.path().join(".mys3").join("config.json")
    }

    /// Whether a bucket exists on the test server.
    pub fn bucket_exists(&self, name: &str) -> bool {
        let response = self
            .client()
            .send("HEAD", &format!("/{name}"), &[], Vec::new(), Vec::new())
            .unwrap_or_else(|err| panic!("HEAD /{name} failed: {err}"));
        response.status == 200
    }

    /// Creates a bucket with a unique name, deleted at the end of the test.
    pub fn new_bucket(&self) -> TempBucket {
        let bucket = self.reserve_bucket();
        let response = bucket.send("PUT", "", &[], Vec::new());
        assert_eq!(
            response.status,
            200,
            "cannot create the test bucket '{}': {}",
            bucket.name,
            String::from_utf8_lossy(&response.body)
        );
        bucket
    }

    /// Picks a unique bucket name without creating the bucket, for the tests
    /// where the command under test creates it. It is deleted at the end of the
    /// test if it exists.
    pub fn reserve_bucket(&self) -> TempBucket {
        TempBucket {
            name: unique_bucket_name(),
            client: self.client(),
        }
    }

    /// A client signed with the keys of the test server.
    fn client(&self) -> S3Client {
        let server = self.server.clone().expect(
            "this test needs the server: build the environment with TestEnv::with_server()",
        );
        S3Client::new(server).expect("invalid test server URL")
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// A bucket of the test server, deleted with its objects when the value is dropped.
pub struct TempBucket {
    name: String,
    client: S3Client,
}

impl TempBucket {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the bucket exists on the server.
    pub fn exists(&self) -> bool {
        self.send("HEAD", "", &[], Vec::new()).status == 200
    }

    /// Stores an object in the bucket, without going through the binary.
    pub fn put_object(&self, key: &str, content: &[u8]) {
        let response = self.send("PUT", key, &[], content.to_vec());
        assert_eq!(
            response.status,
            200,
            "cannot store the object '{key}': {}",
            String::from_utf8_lossy(&response.body)
        );
    }

    /// Content of an object, or `None` if it does not exist.
    pub fn get_object(&self, key: &str) -> Option<Vec<u8>> {
        let response = self.send("GET", key, &[], Vec::new());
        match response.status {
            200 => Some(response.body),
            404 => None,
            status => panic!("cannot read the object '{key}': HTTP {status}"),
        }
    }

    /// Keys of the objects of the bucket, sorted.
    pub fn object_keys(&self) -> Vec<String> {
        self.list_keys()
            .unwrap_or_else(|message| panic!("{message}"))
    }

    /// Sends a request on the bucket (`key` empty) or on one of its objects.
    fn send(&self, method: &str, key: &str, query: &[(&str, &str)], body: Vec<u8>) -> Response {
        self.try_send(method, key, query, body)
            .unwrap_or_else(|message| panic!("{message}"))
    }

    fn try_send(
        &self,
        method: &str,
        key: &str,
        query: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Result<Response, String> {
        let path = if key.is_empty() {
            format!("/{}", self.name)
        } else {
            format!("/{}/{key}", self.name)
        };
        self.client
            .send(method, &path, query, Vec::new(), body)
            .map_err(|err| format!("{method} {path} failed: {err}"))
    }

    /// One page of keys: an empty list when the bucket is empty or does not exist.
    fn list_keys(&self) -> Result<Vec<String>, String> {
        let response = self.try_send("GET", "", &[("list-type", "2")], Vec::new())?;
        match response.status {
            200 => {
                let mut keys = xml_values(&String::from_utf8_lossy(&response.body), "Key");
                keys.sort();
                Ok(keys)
            }
            404 => Ok(Vec::new()),
            status => Err(format!(
                "cannot list the bucket '{}': HTTP {status}",
                self.name
            )),
        }
    }

    /// Deletes the objects, then the bucket. A missing bucket is not an error.
    fn remove(&self) -> Result<(), String> {
        // The listing is paginated: repeat until the bucket is empty.
        loop {
            let keys = self.list_keys()?;
            if keys.is_empty() {
                break;
            }
            for key in keys {
                self.try_send("DELETE", &key, &[], Vec::new())?;
            }
        }
        match self.try_send("DELETE", "", &[], Vec::new())?.status {
            204 | 404 => Ok(()),
            status => Err(format!("HTTP {status}")),
        }
    }
}

impl Drop for TempBucket {
    fn drop(&mut self) {
        // No panic here: a panic while the test is already failing would abort it.
        if let Err(message) = self.remove() {
            eprintln!(
                "warning: cannot delete the test bucket '{}': {message}",
                self.name
            );
        }
    }
}

/// The test server and its keys, read from the environment.
fn server_alias() -> ResolvedAlias {
    let key = |name: &str| {
        std::env::var(name).unwrap_or_else(|_| {
            panic!("{name} is not set: export the MinIO keys before running the ignored tests")
        })
    };
    ResolvedAlias {
        name: TEST_ALIAS.to_string(),
        url: std::env::var("MYS3_TEST_URL").unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string()),
        access_key: key("MYS3_ACCESS_KEY"),
        secret_key: key("MYS3_SECRET_KEY"),
        region: "us-east-1".to_string(),
    }
}

/// A valid bucket name that no other test, in this run or another one, uses.
fn unique_bucket_name() -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is before 1970")
        .as_millis();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mys3-test-{}-{millis}-{count}", std::process::id())
}

/// Text of every `<tag>...</tag>` element of an XML document.
fn xml_values(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut values = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find(&open) {
        rest = &rest[start + open.len()..];
        let Some(end) = rest.find(&close) else {
            break;
        };
        values.push(xml_unescape(&rest[..end]));
        rest = &rest[end + close.len()..];
    }
    values
}

fn xml_unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
