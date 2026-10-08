//! Functional tests for `mys3 list-buckets`.

mod common;

use common::TestEnv;
use predicates::str::contains;

#[test]
fn list_buckets_fails_without_default_alias_configured() {
    let env = TestEnv::new();

    env.cmd()
        .arg("list-buckets")
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn list_buckets_fails_with_nonexistent_alias() {
    let env = TestEnv::new();

    env.cmd()
        .args(["list-buckets", "--alias", "nonexistent"])
        .assert()
        .code(1)
        .stderr(contains("alias 'nonexistent' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn list_buckets_reports_unreachable_server() {
    let env = TestEnv::new();

    // Set an alias pointing to an unreachable port
    env.cmd()
        .args([
            "alias",
            "set",
            "dead",
            "http://127.0.0.1:19999",
            "testkey",
            "testsecret",
        ])
        .assert()
        .success();

    env.cmd()
        .args(["list-buckets", "--alias", "dead"])
        .assert()
        .code(1)
        .stderr(contains("cannot reach server at 'http://127.0.0.1:19999'"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn list_buckets_reports_authentication_refused() {
    let env = TestEnv::new();

    // Set an alias with wrong credentials to real server
    env.cmd()
        .args([
            "alias",
            "set",
            "wrong",
            "http://localhost:9000",
            "wrongkey",
            "wrongsecret",
        ])
        .assert()
        .success();

    env.cmd()
        .env_remove("MYS3_ACCESS_KEY")
        .env_remove("MYS3_SECRET_KEY")
        .args(["list-buckets", "--alias", "wrong"])
        .assert()
        .code(1)
        .stderr(contains("authentication refused by the server"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn list_buckets_shows_created_bucket_in_text_output() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .arg("list-buckets")
        .assert()
        .success()
        .stdout(contains(bucket.name()));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn list_buckets_supports_explicit_alias_flag() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["list-buckets", "--alias", common::TEST_ALIAS])
        .assert()
        .success()
        .stdout(contains(bucket.name()));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn list_buckets_supports_json_output() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    let output = env
        .cmd()
        .args(["list-buckets", "--output", "JSON"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout_str = String::from_utf8_lossy(&output);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_str).expect("output must be valid JSON");

    assert!(parsed.is_array());
    let buckets_array = parsed.as_array().unwrap();
    let found = buckets_array.iter().any(|b| {
        b.get("name").and_then(|n| n.as_str()) == Some(bucket.name())
            && b.get("creation_date").is_some()
    });
    assert!(
        found,
        "bucket '{}' should appear in JSON output",
        bucket.name()
    );
}
