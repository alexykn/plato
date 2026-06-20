use std::path::PathBuf;

use crate::config::{
    Config, GitAutoCrlfConfig, GitAutoCrlfMode, GitEolConfig, GlobalConfig, TemplateEntry,
    parse_global_config_file,
};

#[test]
fn deserializes_global_template_entries() {
    let raw = r#"
[plato]
default_git_provider = "gitlab"

[plato.git_hosts]
gitlab = "gitlab.company.test"

[templates]
py = { path = "~/templates/py" }
api = { git = "gitlab:group/api", rev = "main", subpath = "templates/api" }

[template_configs]
api = "~/.config/plato/template_configs/api.toml"
"#;
    let config: GlobalConfig = toml::from_str(raw).unwrap();

    assert!(matches!(config.templates["py"], TemplateEntry::Path { .. }));
    let TemplateEntry::Git { git, rev, subpath } = &config.templates["api"] else {
        panic!("expected git template");
    };
    assert_eq!(git, "gitlab:group/api");
    assert_eq!(rev.as_deref(), Some("main"));
    assert_eq!(
        subpath.as_deref(),
        Some(PathBuf::from("templates/api").as_path())
    );
    assert_eq!(
        config.template_configs["api"],
        PathBuf::from("~/.config/plato/template_configs/api.toml")
    );
}

#[test]
fn malformed_global_config_fails() {
    let path = std::env::temp_dir().join(format!(
        "plato-bad-global-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, "[templates\n").unwrap();
    let error = parse_global_config_file(&path).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Invalid format in global config")
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn deserializes_template_path_replacements() {
    let raw = r#"
[template.context]
package_name = "py3-requests"
package_deps = "deps-runtime"

[path.replace]
source = { path = "src/py3-something", replace = "src/{{ package_name | regex_replace('^py3-', '') }}" }
deps = { path = "deps/funny", replace = "deps/{{ package_deps | regex_replace('^deps', 'stuff') }}" }
"#;
    let config: Config = toml::from_str(raw).unwrap();

    assert_eq!(
        config.path.replace["source"].path,
        PathBuf::from("src/py3-something")
    );
    assert_eq!(
        config.path.replace["source"].replace,
        "src/{{ package_name | regex_replace('^py3-', '') }}"
    );
    assert_eq!(
        config.path.replace["deps"].path,
        PathBuf::from("deps/funny")
    );
}

#[test]
fn default_template_git_config_does_not_force_local_git_settings() {
    let config: Config = toml::from_str("").unwrap();

    assert_eq!(config.git.initial_branch, None);
    assert_eq!(config.git.user.name, None);
    assert_eq!(config.git.user.email, None);
    assert_eq!(config.git.user.signing_key, None);
    assert_eq!(config.git.commit.gpgsign, None);
    assert!(!config.git.commit.initial);
    assert_eq!(config.git.commit.initial_message, "Initial commit");
    assert_eq!(config.git.core.hooks_path, None);
    assert!(config.git.core.autocrlf.is_none());
    assert!(config.git.core.eol.is_none());
    assert_eq!(config.git.core.filemode, None);
}

#[test]
fn deserializes_template_git_config() {
    let raw = r#"
[git]
initial_branch = "trunk"

[git.user]
name = "Jane Doe"
email = "jane@example.com"
signing_key = "ABC123"

[git.commit]
gpgsign = true
initial = true
initial_message = "Bootstrap project"

[git.core]
hooks_path = ".githooks"
autocrlf = "input"
eol = "lf"
filemode = false
"#;
    let config: Config = toml::from_str(raw).unwrap();

    assert_eq!(config.git.initial_branch.as_deref(), Some("trunk"));
    assert_eq!(config.git.user.name.as_deref(), Some("Jane Doe"));
    assert_eq!(config.git.user.email.as_deref(), Some("jane@example.com"));
    assert_eq!(config.git.user.signing_key.as_deref(), Some("ABC123"));
    assert_eq!(config.git.commit.gpgsign, Some(true));
    assert!(config.git.commit.initial);
    assert_eq!(config.git.commit.initial_message, "Bootstrap project");
    assert_eq!(
        config.git.core.hooks_path.as_deref(),
        Some(PathBuf::from(".githooks").as_path())
    );
    assert!(matches!(
        config.git.core.autocrlf,
        Some(GitAutoCrlfConfig::Mode(GitAutoCrlfMode::Input))
    ));
    assert!(matches!(config.git.core.eol, Some(GitEolConfig::Lf)));
    assert_eq!(config.git.core.filemode, Some(false));
}

#[test]
fn deserializes_template_git_autocrlf_bool() {
    let raw = r"
[git.core]
autocrlf = false
";
    let config: Config = toml::from_str(raw).unwrap();

    assert!(matches!(
        config.git.core.autocrlf,
        Some(GitAutoCrlfConfig::Bool(false))
    ));
}

#[test]
fn deserializes_python_install_config() {
    let raw = r#"
[python]
language_version = "3.12"
package_manager = "uv"
project_scope = "install"

[python.install]
groups = ["dev", "lint"]
extras = ["cli"]
"#;
    let config: Config = toml::from_str(raw).unwrap();

    assert_eq!(config.python.install.groups, ["dev", "lint"]);
    assert_eq!(config.python.install.extras, ["cli"]);
}
