//! Functional tests for `mys3 delete-file`.

mod common;

use common::TestEnv;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn delete_file_requires_bucket_and_key() {
    TestEnv::new()
        .cmd()
        .arg("delete-file")
        .assert()
        .code(1)
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn delete_file_fails_without_default_alias_configured() {
    TestEnv::new()
        .cmd()
        .args(["delete-file", "my-bucket", "file.txt"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn delete_file_fails_with_nonexistent_alias() {
    TestEnv::new()
        .cmd()
        .args([
            "delete-file",
            "my-bucket",
            "file.txt",
            "--alias",
            "nonexistent",
        ])
        .assert()
        .code(1)
        .stderr(contains("alias 'nonexistent' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn delete_file_fails_when_bucket_does_not_exist() {
    let env = TestEnv::with_server();

    env.cmd()
        .args([
            "delete-file",
            "nonexistent-bucket-xyz-12345",
            "file.txt",
            "--force",
        ])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-bucket-xyz-12345' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn delete_file_fails_when_object_does_not_exist() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["delete-file", bucket.name(), "missing.txt", "--force"])
        .assert()
        .code(1)
        .stderr(contains(format!(
            "object 'missing.txt' not found in bucket '{}'",
            bucket.name()
        )));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn delete_file_with_force_deletes_without_asking() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("to_delete.txt", b"data");
    assert!(bucket.get_object("to_delete.txt").is_some());

    env.cmd()
        .args(["delete-file", bucket.name(), "to_delete.txt", "--force"])
        .assert()
        .success()
        .stdout(contains("deleted from bucket"))
        .stderr(contains("[y/N]").not());

    assert!(bucket.get_object("to_delete.txt").is_none());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn delete_file_short_force_flag_works() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("to_delete_short.txt", b"data");

    env.cmd()
        .args(["delete-file", bucket.name(), "to_delete_short.txt", "-f"])
        .assert()
        .success();

    assert!(bucket.get_object("to_delete_short.txt").is_none());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn delete_file_confirms_with_y() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("confirm_y.txt", b"data");

    env.cmd()
        .args(["delete-file", bucket.name(), "confirm_y.txt"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stderr(contains("Delete object 'confirm_y.txt' from bucket"))
        .stdout(contains("deleted from bucket"));

    assert!(bucket.get_object("confirm_y.txt").is_none());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn delete_file_aborts_on_no_or_empty_input() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("keep_me.txt", b"data");

    // Deny confirmation
    env.cmd()
        .args(["delete-file", bucket.name(), "keep_me.txt"])
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(contains("Aborted."));

    assert!(bucket.get_object("keep_me.txt").is_some());

    // Without input (EOF/no terminal)
    env.cmd()
        .args(["delete-file", bucket.name(), "keep_me.txt"])
        .assert()
        .success()
        .stdout(contains("Aborted."));

    assert!(bucket.get_object("keep_me.txt").is_some());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn delete_file_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("alias_delete.txt", b"data");

    env.cmd()
        .args([
            "delete-file",
            bucket.name(),
            "alias_delete.txt",
            "--alias",
            common::TEST_ALIAS,
            "-f",
        ])
        .assert()
        .success();

    assert!(bucket.get_object("alias_delete.txt").is_none());
}
