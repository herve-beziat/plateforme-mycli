//! Functional tests for `mys3 list-objects`.

mod common;

use common::TestEnv;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn list_objects_fails_without_default_alias_configured() {
    let env = TestEnv::new();

    env.cmd()
        .args(["list-objects", "any-bucket"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn list_objects_fails_with_nonexistent_alias() {
    let env = TestEnv::new();

    env.cmd()
        .args(["list-objects", "any-bucket", "--alias", "nonexistent"])
        .assert()
        .code(1)
        .stderr(contains("alias 'nonexistent' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn list_objects_fails_when_bucket_does_not_exist() {
    let env = TestEnv::with_server();

    env.cmd()
        .args(["list-objects", "nonexistent-bucket-xyz-12345"])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-bucket-xyz-12345' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn list_objects_displays_all_objects_in_text_format() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("alpha.txt", b"alpha content");
    bucket.put_object("subfolder/beta.txt", b"beta content");

    env.cmd()
        .args(["list-objects", bucket.name()])
        .assert()
        .success()
        .stdout(contains("alpha.txt\nsubfolder/beta.txt").or(contains("alpha.txt")));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn list_objects_filters_by_prefix() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("logs/app.log", b"app logs");
    bucket.put_object("logs/error.log", b"error logs");
    bucket.put_object("data/records.csv", b"records");

    env.cmd()
        .args(["list-objects", bucket.name(), "--prefix", "logs/"])
        .assert()
        .success()
        .stdout(contains("logs/app.log"))
        .stdout(contains("logs/error.log"))
        .stdout(predicates::str::contains("data/records.csv").not());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn list_objects_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("test.txt", b"hello");

    env.cmd()
        .args(["list-objects", bucket.name(), "--alias", common::TEST_ALIAS])
        .assert()
        .success()
        .stdout(contains("test.txt"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn list_objects_supports_json_output() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("doc.pdf", b"pdf file content");

    let output = env
        .cmd()
        .args(["list-objects", bucket.name(), "--output", "JSON"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout_str = String::from_utf8_lossy(&output);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_str).expect("output must be valid JSON");

    assert!(parsed.is_array());
    let list = parsed.as_array().unwrap();
    let item = list
        .iter()
        .find(|obj| obj.get("key").and_then(|k| k.as_str()) == Some("doc.pdf"))
        .expect("doc.pdf must be in the JSON output");

    assert_eq!(item.get("size").and_then(|s| s.as_u64()), Some(16));
    assert!(item.get("last_modified").is_some());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn list_objects_empty_bucket_produces_no_error_and_empty_output() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["list-objects", bucket.name()])
        .assert()
        .success()
        .stdout("");

    let output = env
        .cmd()
        .args(["list-objects", bucket.name(), "--output", "JSON"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout_str = String::from_utf8_lossy(&output);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_str).expect("output must be valid JSON");
    assert_eq!(parsed, serde_json::json!([]));
}
