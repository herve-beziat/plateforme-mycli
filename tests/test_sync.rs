//! Functional tests for `mys3 sync`.

mod common;

use std::fs;

use common::TestEnv;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn sync_requires_local_folder_and_bucket() {
    TestEnv::new()
        .cmd()
        .arg("sync")
        .assert()
        .code(1)
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn sync_fails_without_default_alias_configured() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    TestEnv::new()
        .cmd()
        .args(["sync", temp_dir.path().to_str().unwrap(), "my-bucket"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn sync_fails_with_nonexistent_alias() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    TestEnv::new()
        .cmd()
        .args([
            "sync",
            temp_dir.path().to_str().unwrap(),
            "my-bucket",
            "--alias",
            "nonexistent",
        ])
        .assert()
        .code(1)
        .stderr(contains("alias 'nonexistent' not found"));
}

#[test]
fn sync_fails_when_local_folder_does_not_exist() {
    let env = TestEnv::with_server();
    let bucket = env.reserve_bucket();

    env.cmd()
        .args([
            "sync",
            "/path/that/does/not/exist/hopefully-12345",
            bucket.name(),
        ])
        .assert()
        .code(1)
        .stderr(contains(
            "local file '/path/that/does/not/exist/hopefully-12345' not found",
        ));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn sync_fails_when_bucket_does_not_exist() {
    let env = TestEnv::with_server();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    env.cmd()
        .args([
            "sync",
            temp_dir.path().to_str().unwrap(),
            "nonexistent-bucket-xyz-98765",
        ])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-bucket-xyz-98765' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn sync_uploads_files_and_nested_subfolders() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    let sub_dir = temp_dir.path().join("sub");
    fs::create_dir_all(&sub_dir).expect("create sub dir");

    fs::write(temp_dir.path().join("file1.txt"), b"hello world").expect("write file1");
    fs::write(sub_dir.join("file2.txt"), b"nested content").expect("write file2");

    env.cmd()
        .args(["sync", temp_dir.path().to_str().unwrap(), bucket.name()])
        .assert()
        .success()
        .stdout(contains("Uploaded 'file1.txt'"))
        .stdout(contains("Uploaded 'sub/file2.txt'"))
        .stdout(contains("Sync complete: 2 uploaded, 0 deleted."));

    assert_eq!(bucket.object_keys(), ["file1.txt", "sub/file2.txt"]);
    assert_eq!(
        bucket.get_object("file1.txt"),
        Some(b"hello world".to_vec())
    );
    assert_eq!(
        bucket.get_object("sub/file2.txt"),
        Some(b"nested content".to_vec())
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn sync_skips_up_to_date_files_and_uploads_only_missing_or_modified() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    fs::write(temp_dir.path().join("same.txt"), b"identical").expect("write same");
    fs::write(temp_dir.path().join("modified.txt"), b"new-larger-content").expect("write mod");

    // Pre-populate bucket
    bucket.put_object("same.txt", b"identical");
    bucket.put_object("modified.txt", b"old");

    env.cmd()
        .args(["sync", temp_dir.path().to_str().unwrap(), bucket.name()])
        .assert()
        .success()
        .stdout(contains("Uploaded 'same.txt'").not())
        .stdout(contains("Uploaded 'modified.txt'"))
        .stdout(contains("Sync complete: 1 uploaded, 0 deleted."));

    assert_eq!(
        bucket.get_object("modified.txt"),
        Some(b"new-larger-content".to_vec())
    );

    // Running again without modifications should report already in sync
    env.cmd()
        .args(["sync", temp_dir.path().to_str().unwrap(), bucket.name()])
        .assert()
        .success()
        .stdout(contains("Folder is already in sync with bucket"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn sync_supports_prefix_flag() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    fs::write(temp_dir.path().join("data.txt"), b"data content").expect("write");

    env.cmd()
        .args([
            "sync",
            temp_dir.path().to_str().unwrap(),
            bucket.name(),
            "--prefix",
            "backup/2026",
        ])
        .assert()
        .success()
        .stdout(contains("Uploaded 'backup/2026/data.txt'"));

    assert_eq!(bucket.object_keys(), ["backup/2026/data.txt"]);
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn sync_dry_run_does_not_modify_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    fs::write(temp_dir.path().join("new_file.txt"), b"content").expect("write");
    bucket.put_object("orphan.txt", b"orphan content");

    env.cmd()
        .args([
            "sync",
            temp_dir.path().to_str().unwrap(),
            bucket.name(),
            "--dry-run",
            "--delete",
        ])
        .assert()
        .success()
        .stdout(contains("Dry run summary:"))
        .stdout(contains("upload: new_file.txt"))
        .stdout(contains("delete: orphan.txt"));

    // Verify nothing changed on the server
    assert_eq!(bucket.object_keys(), ["orphan.txt"]);
    assert!(bucket.get_object("new_file.txt").is_none());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn sync_delete_with_confirmation_and_force() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    fs::write(temp_dir.path().join("keep.txt"), b"keep").expect("write");

    // 1. Abort deletion when answering 'n'
    bucket.put_object("keep.txt", b"keep");
    bucket.put_object("stale.txt", b"stale");

    env.cmd()
        .args([
            "sync",
            temp_dir.path().to_str().unwrap(),
            bucket.name(),
            "--delete",
        ])
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(contains("Deletion aborted. Proceeding with upload only."));

    assert!(bucket.get_object("stale.txt").is_some());

    // 2. Accept deletion when answering 'y'
    env.cmd()
        .args([
            "sync",
            temp_dir.path().to_str().unwrap(),
            bucket.name(),
            "--delete",
        ])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(contains("Deleted 'stale.txt'"))
        .stdout(contains("Sync complete: 0 uploaded, 1 deleted."));

    assert_eq!(bucket.object_keys(), ["keep.txt"]);

    // 3. Deletion with --force / -f skips prompt
    bucket.put_object("another_stale.txt", b"another");

    env.cmd()
        .args([
            "sync",
            temp_dir.path().to_str().unwrap(),
            bucket.name(),
            "--delete",
            "-f",
        ])
        .assert()
        .success()
        .stdout(contains("Deleted 'another_stale.txt'"));

    assert_eq!(bucket.object_keys(), ["keep.txt"]);
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn sync_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    fs::write(temp_dir.path().join("alias_test.txt"), b"alias").expect("write");

    env.cmd()
        .args([
            "sync",
            temp_dir.path().to_str().unwrap(),
            bucket.name(),
            "--alias",
            common::TEST_ALIAS,
        ])
        .assert()
        .success()
        .stdout(contains("Uploaded 'alias_test.txt'"));

    assert_eq!(bucket.object_keys(), ["alias_test.txt"]);
}
