//! Functional tests for `mys3 bucket-info`.

mod common;

use common::TestEnv;
use mys3::commands::bucket_info::count_objects;
use predicates::str::{contains, is_match};
use serde_json::Value;

#[test]
fn bucket_info_fails_without_default_alias_configured() {
    TestEnv::new()
        .cmd()
        .args(["bucket-info", "any-bucket"])
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}

#[test]
fn bucket_info_fails_with_nonexistent_alias() {
    TestEnv::new()
        .cmd()
        .args(["bucket-info", "any-bucket", "--alias", "nonexistent"])
        .assert()
        .code(1)
        .stderr(contains("alias 'nonexistent' not found"));
}

#[test]
fn bucket_info_requires_a_bucket() {
    TestEnv::new()
        .cmd()
        .arg("bucket-info")
        .assert()
        .code(1)
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn bucket_info_refuses_an_unknown_output_format() {
    TestEnv::new()
        .cmd()
        .args(["bucket-info", "b", "--output", "xml"])
        .assert()
        .code(1)
        .stderr(contains("invalid value 'xml'"));
}

#[test]
fn bucket_info_help_lists_alias_and_output() {
    TestEnv::new()
        .cmd()
        .args(["bucket-info", "--help"])
        .assert()
        .success()
        .stdout(contains("--alias"))
        .stdout(contains("--output"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn bucket_info_shows_an_empty_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["bucket-info", bucket.name()])
        .assert()
        .success()
        .stdout(contains(format!("Name:          {}\n", bucket.name())))
        .stdout(is_match(r"Creation date: \d{4}-\d{2}-\d{2}T").unwrap())
        .stdout(contains("Objects:       0\n"))
        .stdout(contains("Total size:    0 bytes\n"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn bucket_info_counts_the_objects_and_sums_their_sizes() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("a.txt", b"hello");
    bucket.put_object("docs/b.bin", &[0; 1024]);

    env.cmd()
        .args(["bucket-info", bucket.name()])
        .assert()
        .success()
        .stdout(contains("Objects:       2\n"))
        .stdout(contains("Total size:    1029 bytes (1.0 KiB)\n"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn bucket_info_supports_json_output() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("a.txt", b"hello");

    let output = env
        .cmd()
        .args(["bucket-info", bucket.name(), "--output", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let info: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(info["name"], bucket.name());
    assert_eq!(info["object_count"], 1);
    assert_eq!(info["total_size"], 5);
    assert!(info["creation_date"].as_str().unwrap().contains('T'));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn bucket_info_supports_explicit_alias() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["bucket-info", bucket.name(), "--alias", common::TEST_ALIAS])
        .assert()
        .success()
        .stdout(contains("Objects:       0\n"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn bucket_info_fails_when_bucket_does_not_exist() {
    let env = TestEnv::with_server();
    let bucket = env.reserve_bucket();

    env.cmd()
        .args(["bucket-info", bucket.name()])
        .assert()
        .code(1)
        .stdout("")
        .stderr(contains(format!("bucket '{}' not found", bucket.name())));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn objects_are_counted_across_several_pages() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    for index in 0..5 {
        bucket.put_object(&format!("file-{index}.txt"), b"abc");
    }

    // 2 objects per page: 3 pages for 5 objects.
    let totals = count_objects(&env.client(), bucket.name(), 2).unwrap();
    assert_eq!(totals, (5, 15));
}
