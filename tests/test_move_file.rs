//! Functional tests for `mys3 move-file`.

mod common;

use common::TestEnv;
use predicates::str::contains;

#[test]
fn move_file_requires_source_bucket_key_and_dest_bucket() {
    TestEnv::new()
        .cmd()
        .arg("move-file")
        .assert()
        .code(1)
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn move_file_fails_without_default_alias_configured() {
    TestEnv::new()
        .cmd()
        .args(["move-file", "src-bkt", "file.txt", "dst-bkt"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn move_file_fails_with_nonexistent_alias() {
    TestEnv::new()
        .cmd()
        .args([
            "move-file",
            "src-bkt",
            "file.txt",
            "dst-bkt",
            "--alias",
            "nonexistent",
        ])
        .assert()
        .code(1)
        .stderr(contains("alias 'nonexistent' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn move_file_fails_when_source_bucket_does_not_exist() {
    let env = TestEnv::with_server();
    let dst_bucket = env.new_bucket();

    env.cmd()
        .args([
            "move-file",
            "nonexistent-src-bkt-12345",
            "file.txt",
            dst_bucket.name(),
        ])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-src-bkt-12345' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn move_file_fails_when_source_object_does_not_exist() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    env.cmd()
        .args([
            "move-file",
            src_bucket.name(),
            "missing.txt",
            dst_bucket.name(),
        ])
        .assert()
        .code(1)
        .stderr(contains(format!(
            "object 'missing.txt' not found in bucket '{}'",
            src_bucket.name()
        )));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn move_file_fails_when_destination_bucket_does_not_exist() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();

    src_bucket.put_object("file.txt", b"source data");

    env.cmd()
        .args([
            "move-file",
            src_bucket.name(),
            "file.txt",
            "nonexistent-dst-bkt-12345",
        ])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-dst-bkt-12345' not found"));

    // Source object must remain untouched
    assert!(src_bucket.get_object("file.txt").is_some());
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn move_file_moves_to_another_bucket_and_deletes_source() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    let content = b"data to move";
    src_bucket.put_object("to_move.pdf", content);

    env.cmd()
        .args([
            "move-file",
            src_bucket.name(),
            "to_move.pdf",
            dst_bucket.name(),
        ])
        .assert()
        .success()
        .stdout(contains("Moved"));

    // Source object is deleted
    assert_eq!(src_bucket.get_object("to_move.pdf"), None);
    // Destination object exists with the content
    assert_eq!(dst_bucket.get_object("to_move.pdf"), Some(content.to_vec()));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn move_file_renames_within_the_same_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    let content = b"content to rename";
    bucket.put_object("old_name.txt", content);

    env.cmd()
        .args([
            "move-file",
            bucket.name(),
            "old_name.txt",
            bucket.name(),
            "--key",
            "new_name.txt",
        ])
        .assert()
        .success();

    // Source object is deleted
    assert_eq!(bucket.get_object("old_name.txt"), None);
    // New object exists
    assert_eq!(bucket.get_object("new_name.txt"), Some(content.to_vec()));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn move_file_refuses_existing_destination_without_overwrite() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    src_bucket.put_object("file.txt", b"source new data");
    dst_bucket.put_object("file.txt", b"destination old data");

    env.cmd()
        .args([
            "move-file",
            src_bucket.name(),
            "file.txt",
            dst_bucket.name(),
        ])
        .assert()
        .code(1)
        .stderr(contains("already exists in bucket"));

    // Neither file changed or was deleted
    assert_eq!(
        src_bucket.get_object("file.txt"),
        Some(b"source new data".to_vec())
    );
    assert_eq!(
        dst_bucket.get_object("file.txt"),
        Some(b"destination old data".to_vec())
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn move_file_replaces_existing_destination_with_overwrite() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    src_bucket.put_object("file.txt", b"source new data");
    dst_bucket.put_object("file.txt", b"destination old data");

    env.cmd()
        .args([
            "move-file",
            src_bucket.name(),
            "file.txt",
            dst_bucket.name(),
            "--overwrite",
        ])
        .assert()
        .success();

    // Source object deleted
    assert_eq!(src_bucket.get_object("file.txt"), None);
    // Destination updated
    assert_eq!(
        dst_bucket.get_object("file.txt"),
        Some(b"source new data".to_vec())
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn move_file_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("item.txt", b"item data");

    env.cmd()
        .args([
            "move-file",
            bucket.name(),
            "item.txt",
            bucket.name(),
            "--key",
            "renamed.txt",
            "--alias",
            common::TEST_ALIAS,
        ])
        .assert()
        .success();

    assert_eq!(bucket.get_object("item.txt"), None);
    assert_eq!(
        bucket.get_object("renamed.txt"),
        Some(b"item data".to_vec())
    );
}
