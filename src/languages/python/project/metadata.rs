use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct PythonProjectMetadata {
    pub(crate) pyproject: PyProjectState,
}

impl PythonProjectMetadata {
    pub(crate) const fn has_project_table(&self) -> bool {
        matches!(self.pyproject, PyProjectState::Present(ref pyproject) if pyproject.has_project_table)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PyProjectState {
    Missing,
    Present(PyProjectTomlMetadata),
}

#[derive(Debug, Clone)]
pub(crate) struct PyProjectTomlMetadata {
    pub(crate) has_project_table: bool,
}

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

pub(crate) fn load_python_project_metadata(root: &Path) -> Result<PythonProjectMetadata> {
    let pyproject_path = Path::new("pyproject.toml");
    let full_pyproject_path = root.join(pyproject_path);
    if !full_pyproject_path.exists() {
        return Ok(PythonProjectMetadata {
            pyproject: PyProjectState::Missing,
        });
    }

    let content = fs::read_to_string(&full_pyproject_path).with_context(|| {
        format!(
            "Could not read rendered pyproject.toml at {}",
            full_pyproject_path.display()
        )
    })?;
    let raw =
        parse_pyproject_content(&content).context("Unable to parse rendered pyproject.toml")?;

    Ok(PythonProjectMetadata {
        pyproject: PyProjectState::Present(PyProjectTomlMetadata {
            has_project_table: raw.project.is_some(),
        }),
    })
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
    fn missing_pyproject_returns_missing_state() {
        let target = make_temp_dir("metadata-missing");
        let metadata = load_python_project_metadata(&target).unwrap();

        assert!(matches!(metadata.pyproject, PyProjectState::Missing));
        assert!(!metadata.has_project_table());
    }

    #[test]
    fn detects_project_table() {
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

        let metadata = load_python_project_metadata(&target).unwrap();

        assert!(metadata.has_project_table());
    }

    #[test]
    fn treats_build_system_only_pyproject_as_no_project_table() {
        let target = make_temp_dir("metadata-build-system-only");
        make_file(
            "pyproject.toml",
            &target,
            r#"
            [build-system]
            requires = []
            build-backend = "backend"
            "#,
        );

        let metadata = load_python_project_metadata(&target).unwrap();

        assert!(!metadata.has_project_table());
    }

    #[test]
    fn invalid_pyproject_returns_error() {
        let target = make_temp_dir("metadata-invalid");
        make_file("pyproject.toml", &target, "not = [valid");

        let error = load_python_project_metadata(&target).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Unable to parse rendered pyproject.toml")
        );
    }
}
