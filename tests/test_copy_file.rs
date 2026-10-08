//! Functional tests for `mys3 copy-file`.

mod common;

use common::TestEnv;
use predicates::str::contains;

#[test]
fn copy_file_requires_source_bucket_key_and_dest_bucket() {
    TestEnv::new()
        .cmd()
        .arg("copy-file")
        .assert()
        .code(1)
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn copy_file_fails_without_default_alias_configured() {
    TestEnv::new()
        .cmd()
        .args(["copy-file", "src-bkt", "file.txt", "dst-bkt"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn copy_file_fails_with_nonexistent_alias() {
    TestEnv::new()
        .cmd()
        .args([
            "copy-file",
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
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_fails_when_source_bucket_does_not_exist() {
    let env = TestEnv::with_server();
    let dst_bucket = env.new_bucket();

    env.cmd()
        .args([
            "copy-file",
            "nonexistent-src-bkt-12345",
            "file.txt",
            dst_bucket.name(),
        ])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-src-bkt-12345' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_fails_when_source_object_does_not_exist() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    env.cmd()
        .args([
            "copy-file",
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
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_fails_when_destination_bucket_does_not_exist() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();

    src_bucket.put_object("file.txt", b"source data");

    env.cmd()
        .args([
            "copy-file",
            src_bucket.name(),
            "file.txt",
            "nonexistent-dst-bkt-12345",
        ])
        .assert()
        .code(1)
        .stderr(contains("bucket 'nonexistent-dst-bkt-12345' not found"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_copies_to_another_bucket_with_same_key_by_default() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    let content = b"data to duplicate";
    src_bucket.put_object("important.pdf", content);

    env.cmd()
        .args([
            "copy-file",
            src_bucket.name(),
            "important.pdf",
            dst_bucket.name(),
        ])
        .assert()
        .success()
        .stdout(contains("Copied"));

    // Source object is preserved
    assert_eq!(
        src_bucket.get_object("important.pdf"),
        Some(content.to_vec())
    );
    // Destination object exists with identical content
    assert_eq!(
        dst_bucket.get_object("important.pdf"),
        Some(content.to_vec())
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_copies_with_custom_destination_key() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    let content = b"custom destination key content";
    src_bucket.put_object("old_name.txt", content);

    env.cmd()
        .args([
            "copy-file",
            src_bucket.name(),
            "old_name.txt",
            dst_bucket.name(),
            "--key",
            "backup/new_name.txt",
        ])
        .assert()
        .success();

    assert_eq!(
        src_bucket.get_object("old_name.txt"),
        Some(content.to_vec())
    );
    assert_eq!(
        dst_bucket.get_object("backup/new_name.txt"),
        Some(content.to_vec())
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_copies_within_the_same_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    let content = b"intra-bucket copy content";
    bucket.put_object("original.txt", content);

    env.cmd()
        .args([
            "copy-file",
            bucket.name(),
            "original.txt",
            bucket.name(),
            "--key",
            "duplicate.txt",
        ])
        .assert()
        .success();

    assert_eq!(bucket.get_object("original.txt"), Some(content.to_vec()));
    assert_eq!(bucket.get_object("duplicate.txt"), Some(content.to_vec()));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_refuses_existing_destination_without_overwrite() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    src_bucket.put_object("file.txt", b"source new data");
    dst_bucket.put_object("file.txt", b"destination old data");

    env.cmd()
        .args([
            "copy-file",
            src_bucket.name(),
            "file.txt",
            dst_bucket.name(),
        ])
        .assert()
        .code(1)
        .stderr(contains("already exists in bucket"));

    // Destination remains unchanged
    assert_eq!(
        dst_bucket.get_object("file.txt"),
        Some(b"destination old data".to_vec())
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_replaces_existing_destination_with_overwrite() {
    let env = TestEnv::with_server();
    let src_bucket = env.new_bucket();
    let dst_bucket = env.new_bucket();

    src_bucket.put_object("file.txt", b"source new data");
    dst_bucket.put_object("file.txt", b"destination old data");

    env.cmd()
        .args([
            "copy-file",
            src_bucket.name(),
            "file.txt",
            dst_bucket.name(),
            "--overwrite",
        ])
        .assert()
        .success();

    assert_eq!(
        dst_bucket.get_object("file.txt"),
        Some(b"source new data".to_vec())
    );
}

#[test]
#[ignore = "needs MinIO (docker compose up -d) and MYS3_ACCESS_KEY / MYS3_SECRET_KEY"]
fn copy_file_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    bucket.put_object("item.txt", b"item data");

    env.cmd()
        .args([
            "copy-file",
            bucket.name(),
            "item.txt",
            bucket.name(),
            "--key",
            "item_copy.txt",
            "--alias",
            common::TEST_ALIAS,
        ])
        .assert()
        .success();

    assert_eq!(
        bucket.get_object("item_copy.txt"),
        Some(b"item data".to_vec())
    );
}
