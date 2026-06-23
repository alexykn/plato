#[cfg(unix)]
#[path = "support/fail_plugin.rs"]
mod fail_plugin;
#[cfg(unix)]
#[path = "support/temp.rs"]
mod temp;

#[cfg(unix)]
use fail_plugin::write_failing_plugin;
#[cfg(unix)]
use temp::TestEnv;

#[cfg(unix)]
#[test]
fn removes_new_target_after_plugin_failure() {
    let env = TestEnv::new("init-cleanup-new");
    write_failing_plugin(&env, "fail");
    env.write(
        "template/plato.toml",
        r#"
        [[setup.steps]]
        plugin = "fail"
        "#,
    );
    env.write("template/README.md", "# Project\n");

    let output = env
        .command()
        .args(["init", "--path", "template", "generated"])
        .output()
        .expect("plato should run");

    assert!(!output.status.success());
    assert!(!env.root().join("generated").exists());
}

#[cfg(unix)]
#[test]
fn preserves_pre_existing_forced_target_after_plugin_failure() {
    let env = TestEnv::new("init-cleanup-force");
    write_failing_plugin(&env, "fail");
    env.write(
        "template/plato.toml",
        r#"
        [[setup.steps]]
        plugin = "fail"
        "#,
    );
    env.write("template/README.md", "# Project\n");
    env.write("generated/sentinel.txt", "keep me\n");

    let output = env
        .command()
        .args(["init", "--path", "template", "--force", "generated"])
        .output()
        .expect("plato should run");

    assert!(!output.status.success());
    assert!(env.root().join("generated").exists());
    assert_eq!(
        std::fs::read_to_string(env.root().join("generated/sentinel.txt")).unwrap(),
        "keep me\n"
    );
}
