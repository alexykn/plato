use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RawPyProject {
    pub(crate) project: Option<RawProjectTable>,
    #[serde(rename = "dependency-groups")]
    pub(crate) dependency_groups: Option<BTreeMap<String, Vec<String>>>,
    pub(crate) tool: Option<RawToolTable>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RawProjectTable {
    pub(crate) dependencies: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RawToolTable {
    pub(crate) uv: Option<RawUvTable>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct RawUvTable {
    pub(crate) dev_dependencies: Option<Vec<String>>,
}

pub(crate) fn parse_pyproject(pyproject_path: &Path) -> Result<RawPyProject> {
    let content = fs::read_to_string(pyproject_path).with_context(|| {
        format!(
            "Could not read pyproject.toml at {}",
            pyproject_path.display()
        )
    })?;
    parse_pyproject_content(&content).with_context(|| {
        format!(
            "Invalid pyproject.toml format at {}",
            pyproject_path.display()
        )
    })
}

fn parse_pyproject_content(content: &str) -> Result<RawPyProject> {
    toml::from_str(content).context("Invalid pyproject.toml format")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(dir_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temp_path = std::env::temp_dir().join(format!("plato-test-{dir_name}-{unique}"));
        fs::create_dir(&temp_path).expect("Failed to create test directory");
        temp_path
    }

    fn make_file(file: &str, path: &Path, content: &str) {
        let file_path = path.join(file);
        fs::write(&file_path, content)
            .unwrap_or_else(|_| panic!("Failed to create file: {}", file_path.display()));
    }

    #[test]
    fn parses_pyproject_metadata() {
        let target = make_temp_dir("metadata-present");
        make_file(
            "pyproject.toml",
            &target,
            r#"
            [project]
            dependencies = []
            readme = "README.md"

            [project.optional-dependencies]
            cli = []

            [project.scripts]
            demo = "demo.cli:main"

            [dependency-groups]
            dev = []
            "#,
        );

        let metadata = parse_pyproject(&target.join("pyproject.toml")).unwrap();

        assert!(metadata.project.is_some());
        assert!(metadata.dependency_groups.unwrap().contains_key("dev"));
    }

    #[test]
    fn invalid_pyproject_returns_error() {
        let target = make_temp_dir("metadata-invalid");
        make_file("pyproject.toml", &target, "not = [valid");

        let error = parse_pyproject(&target.join("pyproject.toml")).unwrap_err();

        assert!(error.to_string().contains("Invalid pyproject.toml format"));
    }
}
