mod common;

use common::TestEnv;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn test_unknown_alias_is_refused() {
    TestEnv::new()
        .cmd()
        .args(["delete-bucket", "my-bucket", "--force", "--alias", "nope"])
        .assert()
        .code(1)
        .stderr(contains("alias 'nope' not found"));
}

#[test]
#[ignore = "needs MinIO"]
fn test_force_deletes_an_empty_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["delete-bucket", bucket.name(), "--force"])
        .assert()
        .success()
        .stdout(contains(format!("Bucket '{}' deleted.", bucket.name())))
        .stderr(contains("[y/N]").not());

    assert!(!bucket.exists());
}

#[test]
#[ignore = "needs MinIO"]
fn test_short_force_flag() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["delete-bucket", bucket.name(), "-f"])
        .assert()
        .success()
        .stderr(contains("[y/N]").not());

    assert!(!bucket.exists());
}

#[test]
#[ignore = "needs MinIO"]
fn test_confirmed_deletion() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["delete-bucket", bucket.name()])
        .write_stdin("y\n")
        .assert()
        .success()
        .stderr(contains(format!(
            "Delete bucket '{}'? [y/N]",
            bucket.name()
        )));

    assert!(!bucket.exists());
}

#[test]
#[ignore = "needs MinIO"]
fn test_refused_confirmation_keeps_the_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    for typed in ["n\n", "no\n", "\n"] {
        env.cmd()
            .args(["delete-bucket", bucket.name()])
            .write_stdin(typed)
            .assert()
            .success()
            .stdout(contains("Aborted."));

        assert!(bucket.exists(), "answer {typed:?} should keep the bucket");
    }
}

/// Without `write_stdin` there is no input at all, as in a script.
#[test]
#[ignore = "needs MinIO"]
fn test_no_input_keeps_the_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();

    env.cmd()
        .args(["delete-bucket", bucket.name()])
        .assert()
        .success()
        .stdout(contains("Aborted."));

    assert!(bucket.exists());
}

#[test]
#[ignore = "needs MinIO"]
fn test_missing_bucket_is_refused() {
    let env = TestEnv::with_server();
    let bucket = env.reserve_bucket();

    env.cmd()
        .args(["delete-bucket", bucket.name()])
        .assert()
        .code(1)
        .stderr(contains(format!("bucket '{}' not found", bucket.name())))
        .stderr(contains("[y/N]").not());
}

#[test]
#[ignore = "needs MinIO"]
fn test_non_empty_bucket_is_refused_without_recursive() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("file.txt", b"content");

    env.cmd()
        .args(["delete-bucket", bucket.name(), "--force"])
        .assert()
        .code(1)
        .stderr(contains(format!("bucket '{}' is not empty", bucket.name())));

    assert!(bucket.exists());
    assert_eq!(bucket.object_keys(), ["file.txt"]);
}

#[test]
#[ignore = "needs MinIO"]
fn test_recursive_deletes_the_objects_and_the_bucket() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("a.txt", b"a");
    bucket.put_object("b.txt", b"b");
    bucket.put_object("subfolder/c.txt", b"c");

    env.cmd()
        .args(["delete-bucket", bucket.name(), "--force", "--recursive"])
        .assert()
        .success()
        .stdout(contains(format!("Bucket '{}' deleted.", bucket.name())));

    assert!(!bucket.exists());
}

#[test]
#[ignore = "needs MinIO"]
fn test_recursive_asks_a_single_question() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("a.txt", b"a");

    env.cmd()
        .args(["delete-bucket", bucket.name(), "--recursive"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(contains(format!("Bucket '{}' deleted.", bucket.name())));

    assert!(!bucket.exists());
}

#[test]
#[ignore = "needs MinIO"]
fn test_recursive_refusal_keeps_the_objects() {
    let env = TestEnv::with_server();
    let bucket = env.new_bucket();
    bucket.put_object("a.txt", b"a");

    env.cmd()
        .args(["delete-bucket", bucket.name(), "--recursive"])
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(contains("Aborted."));

    assert_eq!(bucket.object_keys(), ["a.txt"]);
}
