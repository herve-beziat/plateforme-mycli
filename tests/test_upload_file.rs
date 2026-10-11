//! Functional tests for `mys3 upload-file`.

mod common;

use std::path::{Path, PathBuf};

use common::TestEnv;
use predicates::str::contains;

/// Writes `content` to `dir/name` and returns its path.
fn local_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn upload_file_requires_file_and_bucket() {
    TestEnv::new()
        .cmd()
        .arg("upload-file")
        .assert()
        .code(1)
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn upload_file_fails_when_local_file_does_not_exist() {
    TestEnv::new()
        .cmd()
        .args(["upload-file", "missing-file.txt", "my-bucket"])
        .assert()
        .code(1)
        .stderr(contains("local file 'missing-file.txt' not found"));
}

#[test]
fn upload_file_refuses_a_directory() {
    let temp_dir = tempfile::tempdir().unwrap();

    TestEnv::new()
        .cmd()
        .args([
            "upload-file",
            temp_dir.path().to_str().unwrap(),
            "my-bucket",
        ])
        .assert()
        .code(1)
        .stderr(contains("not found"));
}

#[test]
fn upload_file_fails_without_default_alias_configured() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file = local_file(temp_dir.path(), "file.txt", b"content");

    TestEnv::new()
        .cmd()
        .args(["upload-file", file.to_str().unwrap(), "my-bucket"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn upload_file_fails_with_nonexistent_alias() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file = local_file(temp_dir.path(), "file.txt", b"content");

    TestEnv::new()
        .cmd()
        .args([
            "upload-file",
            file.to_str().unwrap(),
            "my-bucket",
            "--alias",
            "nonexistent",
        ])
        .assert()
        .code(1)
        .stderr(contains("alias 'nonexistent' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn upload_file_fails_when_bucket_does_not_exist() {
    let env = TestEnv::with_server();
    let temp_dir = tempfile::tempdir().unwrap();
    let file = local_file(temp_dir.path(), "file.txt", b"content");

    env.cmd()
        .args([
            "upload-file",
            file.to_str().unwrap(),
            "nonexistent-bucket-xyz-12345",
        ])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-bucket-xyz-12345' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn upload_file_uses_the_file_name_as_key_by_default() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().unwrap();
    let file = local_file(temp_dir.path(), "report.txt", b"hello from upload test");

    env.cmd()
        .args(["upload-file", file.to_str().unwrap(), bucket.name()])
        .assert()
        .success()
        .stdout(contains(format!(
            "to bucket '{}' as 'report.txt'",
            bucket.name()
        )));

    assert_eq!(
        bucket.get_object("report.txt").as_deref(),
        Some(&b"hello from upload test"[..])
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn upload_file_uses_the_explicit_key() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().unwrap();
    let file = local_file(temp_dir.path(), "report.txt", b"nested content");

    env.cmd()
        .args([
            "upload-file",
            file.to_str().unwrap(),
            bucket.name(),
            "--key",
            "archive/2026-report.txt",
        ])
        .assert()
        .success();

    assert_eq!(bucket.object_keys(), vec!["archive/2026-report.txt"]);
    assert_eq!(
        bucket.get_object("archive/2026-report.txt").as_deref(),
        Some(&b"nested content"[..])
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn upload_file_refuses_existing_object_without_overwrite() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("file.txt", b"old content");
    let temp_dir = tempfile::tempdir().unwrap();
    let file = local_file(temp_dir.path(), "file.txt", b"new content");

    env.cmd()
        .args(["upload-file", file.to_str().unwrap(), bucket.name()])
        .assert()
        .code(1)
        .stderr(contains(format!(
            "object 'file.txt' already exists in bucket '{}'",
            bucket.name()
        )));

    assert_eq!(
        bucket.get_object("file.txt").as_deref(),
        Some(&b"old content"[..])
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn upload_file_replaces_existing_object_with_overwrite() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("file.txt", b"old content");
    let temp_dir = tempfile::tempdir().unwrap();
    let file = local_file(temp_dir.path(), "file.txt", b"new content");

    env.cmd()
        .args([
            "upload-file",
            file.to_str().unwrap(),
            bucket.name(),
            "--overwrite",
        ])
        .assert()
        .success();

    assert_eq!(
        bucket.get_object("file.txt").as_deref(),
        Some(&b"new content"[..])
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn upload_file_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().unwrap();
    let file = local_file(temp_dir.path(), "alias.txt", b"alias test content");

    env.cmd()
        .args([
            "upload-file",
            file.to_str().unwrap(),
            bucket.name(),
            "--alias",
            common::TEST_ALIAS,
        ])
        .assert()
        .success();

    assert_eq!(
        bucket.get_object("alias.txt").as_deref(),
        Some(&b"alias test content"[..])
    );
}
