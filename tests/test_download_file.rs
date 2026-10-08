//! Functional tests for `mys3 download-file`.

mod common;

use common::TestEnv;
use predicates::str::contains;

#[test]
fn download_file_requires_bucket_and_key() {
    TestEnv::new()
        .cmd()
        .arg("download-file")
        .assert()
        .code(1)
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn download_file_fails_without_default_alias_configured() {
    TestEnv::new()
        .cmd()
        .args(["download-file", "my-bucket", "file.txt"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn download_file_fails_with_nonexistent_alias() {
    TestEnv::new()
        .cmd()
        .args([
            "download-file",
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
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn download_file_fails_when_bucket_does_not_exist() {
    let env = TestEnv::with_server();

    env.cmd()
        .args(["download-file", "nonexistent-bucket-xyz-12345", "file.txt"])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-bucket-xyz-12345' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn download_file_fails_when_object_does_not_exist() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["download-file", bucket.name(), "missing.txt"])
        .assert()
        .code(1)
        .stderr(contains(format!(
            "object 'missing.txt' not found in bucket '{}'",
            bucket.name()
        )));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn download_file_saves_to_current_directory_by_default() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    let content = b"hello from s3 download test";
    bucket.put_object("test_default.txt", content);

    let temp_dir = tempfile::tempdir().unwrap();

    env.cmd()
        .current_dir(temp_dir.path())
        .args(["download-file", bucket.name(), "test_default.txt"])
        .assert()
        .success()
        .stdout(contains("Downloaded 'test_default.txt'"));

    let downloaded_path = temp_dir.path().join("test_default.txt");
    assert!(downloaded_path.exists());
    assert_eq!(std::fs::read(downloaded_path).unwrap(), content);
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn download_file_extracts_basename_for_nested_keys() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    let content = b"nested file content";
    bucket.put_object("path/to/my_nested_file.txt", content);

    let temp_dir = tempfile::tempdir().unwrap();

    env.cmd()
        .current_dir(temp_dir.path())
        .args(["download-file", bucket.name(), "path/to/my_nested_file.txt"])
        .assert()
        .success();

    let downloaded_path = temp_dir.path().join("my_nested_file.txt");
    assert!(downloaded_path.exists());
    assert_eq!(std::fs::read(downloaded_path).unwrap(), content);
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn download_file_saves_to_explicit_output_path() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    let content = b"custom destination content";
    bucket.put_object("remote.bin", content);

    let temp_dir = tempfile::tempdir().unwrap();
    let target_file = temp_dir.path().join("sub/custom_name.bin");

    env.cmd()
        .args([
            "download-file",
            bucket.name(),
            "remote.bin",
            "--output",
            target_file.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(target_file.exists());
    assert_eq!(std::fs::read(target_file).unwrap(), content);
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn download_file_refuses_existing_file_without_overwrite() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("file.txt", b"new content");

    let temp_dir = tempfile::tempdir().unwrap();
    let existing_file = temp_dir.path().join("file.txt");
    std::fs::write(&existing_file, b"old content").unwrap();

    env.cmd()
        .args([
            "download-file",
            bucket.name(),
            "file.txt",
            "--output",
            existing_file.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stderr(contains("already exists (use --overwrite to replace it)"));

    // File content should not have been overwritten
    assert_eq!(std::fs::read(&existing_file).unwrap(), b"old content");
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn download_file_replaces_existing_file_with_overwrite() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("file.txt", b"new content");

    let temp_dir = tempfile::tempdir().unwrap();
    let existing_file = temp_dir.path().join("file.txt");
    std::fs::write(&existing_file, b"old content").unwrap();

    env.cmd()
        .args([
            "download-file",
            bucket.name(),
            "file.txt",
            "--output",
            existing_file.to_str().unwrap(),
            "--overwrite",
        ])
        .assert()
        .success();

    assert_eq!(std::fs::read(&existing_file).unwrap(), b"new content");
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn download_file_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("alias_test.txt", b"alias test content");

    let temp_dir = tempfile::tempdir().unwrap();
    let target = temp_dir.path().join("downloaded.txt");

    env.cmd()
        .args([
            "download-file",
            bucket.name(),
            "alias_test.txt",
            "--output",
            target.to_str().unwrap(),
            "--alias",
            common::TEST_ALIAS,
        ])
        .assert()
        .success();

    assert_eq!(std::fs::read(target).unwrap(), b"alias test content");
}
