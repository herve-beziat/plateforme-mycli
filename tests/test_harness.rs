//! Tests of the shared test helpers (`tests/common`), which also show how to
//! use them in the tests of a command.

mod common;

use common::TestEnv;
use predicates::str::contains;

#[test]
fn binary_runs_and_lists_its_commands() {
    TestEnv::new()
        .cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("list-buckets"));
}

#[test]
fn usage_error_goes_to_stderr_with_exit_code_1() {
    TestEnv::new()
        .cmd()
        .arg("no-such-command")
        .assert()
        .code(1)
        .stdout("")
        .stderr(contains("unrecognized subcommand"));
}

#[test]
fn configuration_is_written_in_the_temporary_home() {
    let env = TestEnv::new();
    assert!(!env.config_path().exists());

    env.cmd()
        .args([
            "alias",
            "set",
            "local",
            "http://localhost:9000",
            "access",
            "secret",
        ])
        .assert()
        .success();

    let config = std::fs::read_to_string(env.config_path()).unwrap();
    assert!(config.contains("\"local\""));
}

#[test]
fn each_environment_has_its_own_home() {
    let first = TestEnv::new();
    let second = TestEnv::new();
    assert_ne!(first.home(), second.home());

    first
        .cmd()
        .args([
            "alias",
            "set",
            "local",
            "http://localhost:9000",
            "access",
            "secret",
        ])
        .assert()
        .success();

    assert!(first.config_path().exists());
    assert!(!second.config_path().exists());
}

#[test]
fn dotenv_reads_key_value_lines() {
    let settings = common::parse_dotenv("MINIO_ROOT_USER=admin\nMINIO_PORT=9000\n");
    assert_eq!(settings["MINIO_ROOT_USER"], "admin");
    assert_eq!(settings["MINIO_PORT"], "9000");
}

#[test]
fn dotenv_ignores_comments_and_blank_lines() {
    let settings = common::parse_dotenv("# Local MinIO\n\n   \nMINIO_PORT=9000\n");
    assert_eq!(settings.len(), 1);
    assert_eq!(settings["MINIO_PORT"], "9000");
}

#[test]
fn dotenv_removes_quotes_and_keeps_equal_signs_in_values() {
    let settings = common::parse_dotenv("A=\"quoted\"\nB='single'\nC=x=y\n  D = spaced  \n");
    assert_eq!(settings["A"], "quoted");
    assert_eq!(settings["B"], "single");
    assert_eq!(settings["C"], "x=y");
    assert_eq!(settings["D"], "spaced");
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn server_environment_has_a_default_alias() {
    let env = TestEnv::with_server();

    let config = std::fs::read_to_string(env.config_path()).unwrap();
    assert!(config.contains(&format!("\"default\": \"{}\"", common::TEST_ALIAS)));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn signed_list_buckets_is_accepted_by_minio() {
    let response = TestEnv::with_server()
        .client()
        .send("GET", "/", &[], Vec::new(), Vec::new())
        .unwrap();
    assert_eq!(response.status, 200);
    assert!(String::from_utf8_lossy(&response.body).contains("ListAllMyBucketsResult"));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn new_bucket_exists_then_is_deleted_with_its_objects() {
    let env = TestEnv::with_server();
    let name;
    {
        let bucket = env.new_bucket();
        name = bucket.name().to_string();
        assert!(bucket.exists());

        bucket.put_object("notes/hello world.txt", b"hello");
        bucket.put_object("data.bin", &[0, 1, 2]);
        assert_eq!(bucket.object_keys(), ["data.bin", "notes/hello world.txt"]);
        assert_eq!(bucket.get_object("data.bin"), Some(vec![0, 1, 2]));
        assert_eq!(bucket.get_object("missing"), None);
    } // the bucket is dropped here

    assert!(!env.bucket_exists(&name));
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn bucket_names_are_unique_and_valid() {
    let env = TestEnv::with_server();
    let first = env.reserve_bucket();
    let second = env.reserve_bucket();

    assert_ne!(first.name(), second.name());
    for name in [first.name(), second.name()] {
        assert!(name.len() <= 63);
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        );
    }
}

#[test]
#[ignore = "needs MinIO (docker compose up -d)"]
fn reserved_bucket_is_not_created() {
    let env = TestEnv::with_server();
    let bucket = env.reserve_bucket();
    assert!(!bucket.exists());
}
