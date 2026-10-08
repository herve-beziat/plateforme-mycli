//! Functional tests for `mys3 object-info`.

mod common;

use common::TestEnv;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::{contains, is_match};
use serde_json::Value;

const MINIO: &str = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY";

#[test]
fn object_info_fails_without_default_alias_configured() {
    TestEnv::new()
        .cmd()
        .args(["object-info", "any-bucket", "key"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn object_info_fails_with_nonexistent_alias() {
    TestEnv::new()
        .cmd()
        .args(["object-info", "any-bucket", "key", "--alias", "nonexistent"])
        .assert()
        .code(1)
        .stderr(contains("alias 'nonexistent' not found"));
}

#[test]
fn object_info_requires_a_bucket_and_a_key() {
    TestEnv::new()
        .cmd()
        .arg("object-info")
        .assert()
        .code(1)
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn object_info_refuses_an_unknown_output_format() {
    TestEnv::new()
        .cmd()
        .args(["object-info", "b", "k", "--output", "xml"])
        .assert()
        .code(1)
        .stderr(contains("invalid value 'xml'"));
}

#[test]
fn object_info_refuses_an_empty_key_before_any_request() {
    TestEnv::new()
        .cmd()
        .args(["object-info", "my-bucket", ""])
        .assert()
        .code(1)
        .stderr(contains("object '' not found in bucket 'my-bucket'"));
}

#[test]
fn object_info_help_lists_alias_and_output() {
    TestEnv::new()
        .cmd()
        .args(["object-info", "--help"])
        .assert()
        .success()
        .stdout(contains("--alias"))
        .stdout(contains("--output"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn object_info_displays_the_details_in_text_format() {
    let _ = MINIO;
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object_with_content_type("docs/report.txt", b"hello world", "text/plain");

    env.cmd()
        .args(["object-info", bucket.name(), "docs/report.txt"])
        .assert()
        .success()
        .stdout(contains("docs/report.txt"))
        .stdout(contains("11 bytes"))
        .stdout(contains("text/plain"))
        .stdout(contains("5eb63bbbe01eeed093cb22bb8f5acdc3"))
        .stdout(is_match(r"Last modified: +\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z").unwrap())
        .stdout(contains("\"").not());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn object_info_supports_json_output() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object_with_content_type("a.txt", b"hello world", "text/plain");

    for format in ["JSON", "json"] {
        let output = env
            .cmd()
            .args(["object-info", bucket.name(), "a.txt", "--output", format])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let json: Value = serde_json::from_slice(&output).expect("output must be valid JSON");
        assert_eq!(json.as_object().unwrap().len(), 5);
        assert_eq!(json["name"], "a.txt");
        assert_eq!(json["size"], 11);
        assert_eq!(json["content_type"], "text/plain");
        assert_eq!(json["etag"], "5eb63bbbe01eeed093cb22bb8f5acdc3");
        chrono::DateTime::parse_from_rfc3339(json["last_modified"].as_str().unwrap())
            .expect("last_modified must be RFC 3339");
    }
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn object_info_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("hello.txt", b"hello");

    env.cmd()
        .args([
            "alias",
            "set",
            "dead",
            "http://127.0.0.1:19999",
            "k",
            "s",
            "--default",
        ])
        .assert()
        .success();
    env.cmd()
        .args(["object-info", bucket.name(), "hello.txt"])
        .assert()
        .code(1)
        .stderr(contains("cannot reach server at 'http://127.0.0.1:19999'"));
    env.cmd()
        .args([
            "object-info",
            bucket.name(),
            "hello.txt",
            "--alias",
            common::TEST_ALIAS,
        ])
        .assert()
        .success()
        .stdout(contains("hello.txt"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn object_info_reports_an_empty_object() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("empty", b"");

    env.cmd()
        .args(["object-info", bucket.name(), "empty"])
        .assert()
        .success()
        .stdout(contains("0 bytes"))
        .stdout(contains("d41d8cd98f00b204e9800998ecf8427e"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn object_info_supports_special_characters_in_keys() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let key = "reports/2026 summary (final) é.txt";
    bucket.put_object(key, b"x");

    env.cmd()
        .args(["object-info", bucket.name(), key])
        .assert()
        .success()
        .stdout(contains(key))
        .stdout(contains("1 byte"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn object_info_fails_when_bucket_does_not_exist() {
    let env = TestEnv::with_server();

    env.cmd()
        .args(["object-info", "nonexistent-bucket-xyz-12345", "k"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(contains("bucket 'nonexistent-bucket-xyz-12345' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn object_info_fails_when_object_does_not_exist() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["object-info", bucket.name(), "missing.txt"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(contains(format!(
            "object 'missing.txt' not found in bucket '{}'",
            bucket.name()
        )));
}
