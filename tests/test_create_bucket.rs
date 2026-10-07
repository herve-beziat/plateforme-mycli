mod common;

use common::TestEnv;
use mys3::commands::create_bucket::{location_body, validate_bucket_name};
use mys3::error::MyS3Error;
use predicates::str::contains;

#[test]
fn test_valid_bucket_names() {
    for name in ["my-bucket", "abc", "my.bucket.01", &"a".repeat(63)] {
        assert!(
            validate_bucket_name(name).is_ok(),
            "'{name}' should be valid"
        );
    }
}

#[test]
fn test_invalid_bucket_names() {
    for name in [
        "ab",
        &"a".repeat(64),
        "My-Bucket",
        "my_bucket",
        "-bucket",
        "bucket-",
        "my..bucket",
        "192.168.0.1",
        "xn--bucket",
        "bucket-s3alias",
    ] {
        assert!(
            matches!(
                validate_bucket_name(name),
                Err(MyS3Error::InvalidBucketName(_))
            ),
            "'{name}' should be invalid"
        );
    }
}

#[test]
fn test_location_body_is_empty_for_us_east_1() {
    assert!(location_body("us-east-1").is_empty());
}

#[test]
fn test_location_body_contains_the_region() {
    let body = String::from_utf8(location_body("eu-west-1")).unwrap();
    assert!(body.contains("<LocationConstraint>eu-west-1</LocationConstraint>"));
}

/// The name is checked before the configuration is read: no alias is needed.
#[test]
fn test_invalid_name_is_refused_before_any_request() {
    TestEnv::new()
        .cmd()
        .args(["create-bucket", "Bad_Name"])
        .assert()
        .code(1)
        .stderr(contains("invalid bucket name 'Bad_Name'"));
}

#[test]
fn test_unknown_alias_is_refused() {
    TestEnv::new()
        .cmd()
        .args(["create-bucket", "my-bucket", "--alias", "nope"])
        .assert()
        .code(1)
        .stderr(contains("alias 'nope' not found"));
}

#[test]
#[ignore = "needs MinIO"]
fn test_create_bucket_creates_the_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.reserve_bucket();

    env.cmd()
        .args(["create-bucket", bucket.name()])
        .assert()
        .success()
        .stdout(contains(format!("Bucket '{}' created.", bucket.name())));

    assert!(bucket.exists());
}

#[test]
#[ignore = "needs MinIO"]
fn test_create_bucket_with_region() {
    let env = TestEnv::with_server();
    let bucket = env.reserve_bucket();

    env.cmd()
        .args(["create-bucket", bucket.name(), "--region", "eu-west-1"])
        .assert()
        .success();

    assert!(bucket.exists());
}

#[test]
#[ignore = "needs MinIO"]
fn test_existing_bucket_is_refused() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["create-bucket", bucket.name()])
        .assert()
        .code(1)
        .stderr(contains(format!(
            "bucket '{}' already exists",
            bucket.name()
        )));
}
