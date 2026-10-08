//! Functional tests for `mys3 alias list`.

mod common;

use common::TestEnv;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use serde_json::Value;

/// Saves an alias through the binary, as a user would.
fn alias_set(env: &TestEnv, args: &[&str]) {
    env.cmd()
        .args(["alias", "set"])
        .args(args)
        .assert()
        .success();
}

/// `alias list --output JSON`, parsed.
fn list_json(env: &TestEnv) -> Vec<Value> {
    let output = env
        .cmd()
        .args(["alias", "list", "--output", "JSON"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("alias list --output JSON must print JSON")
}

#[test]
fn no_alias_is_a_message_not_an_error() {
    TestEnv::new()
        .cmd()
        .args(["alias", "list"])
        .assert()
        .success()
        .stdout(contains("No aliases configured"));
}

#[test]
fn no_alias_gives_an_empty_json_list() {
    assert!(list_json(&TestEnv::new()).is_empty());
}

#[test]
fn lists_name_url_region_and_marks_the_default() {
    let env = TestEnv::new();
    alias_set(
        &env,
        &["local", "http://localhost:9000", "access", "secret"],
    );
    alias_set(
        &env,
        &[
            "prod",
            "https://s3.example.com",
            "access",
            "secret",
            "--region",
            "eu-west-1",
        ],
    );

    env.cmd()
        .args(["alias", "list"])
        .assert()
        .success()
        .stdout(contains("* local"))
        .stdout(contains("http://localhost:9000"))
        .stdout(contains("  prod"))
        .stdout(contains("https://s3.example.com"))
        .stdout(contains("eu-west-1"));
}

#[test]
fn never_displays_the_keys() {
    let env = TestEnv::new();
    alias_set(
        &env,
        &[
            "local",
            "http://localhost:9000",
            "my-access-key",
            "my-secret-key",
        ],
    );

    for output in ["text", "JSON"] {
        env.cmd()
            .args(["alias", "list", "--output", output])
            .assert()
            .success()
            .stdout(contains("my-access-key").not())
            .stdout(contains("my-secret-key").not());
    }
}

#[test]
fn json_output_lists_every_alias() {
    let env = TestEnv::new();
    alias_set(
        &env,
        &["local", "http://localhost:9000", "access", "secret"],
    );
    alias_set(
        &env,
        &["prod", "https://s3.example.com", "access", "secret"],
    );

    let aliases = list_json(&env);
    assert_eq!(aliases.len(), 2);
    assert_eq!(aliases[0]["name"], "local");
    assert_eq!(aliases[0]["url"], "http://localhost:9000");
    assert_eq!(aliases[0]["region"], "us-east-1");
    assert_eq!(aliases[0]["default"], true);
    assert_eq!(aliases[1]["name"], "prod");
    assert_eq!(aliases[1]["default"], false);
}

#[test]
fn default_marker_follows_alias_use() {
    let env = TestEnv::new();
    alias_set(
        &env,
        &["local", "http://localhost:9000", "access", "secret"],
    );
    alias_set(
        &env,
        &["prod", "https://s3.example.com", "access", "secret"],
    );

    env.cmd().args(["alias", "use", "prod"]).assert().success();

    env.cmd()
        .args(["alias", "list"])
        .assert()
        .success()
        .stdout(contains("* prod"))
        .stdout(contains("  local"));
}
