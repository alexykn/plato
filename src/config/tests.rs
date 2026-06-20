use std::path::PathBuf;

use crate::config::{Config, GlobalConfig, TemplateEntry, parse_global_config_file};

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
