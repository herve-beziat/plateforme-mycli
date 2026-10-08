//! Functional tests for `mys3 alias remove`.

mod common;

use common::TestEnv;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

/// An environment with two aliases: `local` (the default) and `prod`.
fn env_with_two_aliases() -> TestEnv {
    let env = TestEnv::new();
    for (name, url) in [
        ("local", "http://localhost:9000"),
        ("prod", "https://s3.example.com"),
    ] {
        env.cmd()
            .args(["alias", "set", name, url, "access", "secret"])
            .assert()
            .success();
    }
    env
}

/// Output of `alias list`.
fn alias_list(env: &TestEnv) -> String {
    let output = env.cmd().args(["alias", "list"]).assert().success();
    String::from_utf8(output.get_output().stdout.clone()).unwrap()
}

#[test]
fn force_removes_the_alias_without_asking() {
    let env = env_with_two_aliases();

    env.cmd()
        .args(["alias", "remove", "prod", "--force"])
        .assert()
        .success()
        .stdout(contains("Alias 'prod' removed."))
        .stderr(contains("[y/N]").not());

    assert!(!alias_list(&env).contains("prod"));
}

#[test]
fn short_force_flag_works() {
    let env = env_with_two_aliases();

    env.cmd()
        .args(["alias", "remove", "prod", "-f"])
        .assert()
        .success();

    assert!(!alias_list(&env).contains("prod"));
}

#[test]
fn yes_answer_removes_the_alias() {
    let env = env_with_two_aliases();

    env.cmd()
        .args(["alias", "remove", "prod"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stderr(contains("Remove alias 'prod'? [y/N]"))
        .stdout(contains("Alias 'prod' removed."));

    assert!(!alias_list(&env).contains("prod"));
}

#[test]
fn any_other_answer_keeps_the_alias() {
    for typed in ["n\n", "no\n", "\n", "maybe\n"] {
        let env = env_with_two_aliases();

        env.cmd()
            .args(["alias", "remove", "prod"])
            .write_stdin(typed)
            .assert()
            .success()
            .stdout(contains("Aborted."));

        assert!(alias_list(&env).contains("prod"), "{typed:?} must keep it");
    }
}

#[test]
fn no_terminal_keeps_the_alias() {
    let env = env_with_two_aliases();

    // Without --force and without any input, nothing is removed.
    env.cmd()
        .args(["alias", "remove", "prod"])
        .assert()
        .success()
        .stdout(contains("Aborted."));

    assert!(alias_list(&env).contains("prod"));
}

#[test]
fn unknown_alias_is_an_error_without_question() {
    let env = env_with_two_aliases();

    env.cmd()
        .args(["alias", "remove", "nope"])
        .assert()
        .code(1)
        .stderr(contains("alias 'nope' not found"))
        .stderr(contains("[y/N]").not());
}

#[test]
fn removing_the_default_alias_leaves_no_default() {
    let env = env_with_two_aliases();

    env.cmd()
        .args(["alias", "remove", "local", "--force"])
        .assert()
        .success()
        .stdout(contains("mys3 alias use"));

    let list = alias_list(&env);
    assert!(list.contains("prod"));
    assert!(!list.contains('*'), "no alias may be marked as default");

    env.cmd()
        .arg("list-buckets")
        .assert()
        .code(1)
        .stderr(contains("no default alias set"));
}
