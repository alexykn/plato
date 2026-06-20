use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::validation::files::ProjectFiles;

#[derive(Debug, Clone)]
pub(crate) struct PythonProjectMetadata {
    pub(crate) pyproject: PyProjectState,
}

impl PythonProjectMetadata {
    pub(crate) const fn has_project_table(&self) -> bool {
        matches!(self.pyproject, PyProjectState::Present(ref pyproject) if pyproject.has_project_table)
    }

    pub(crate) const fn pyproject(&self) -> Option<&PyProjectTomlMetadata> {
        match &self.pyproject {
            PyProjectState::Missing => None,
            PyProjectState::Present(pyproject) => Some(pyproject),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PyProjectState {
    Missing,
    Present(PyProjectTomlMetadata),
}

#[derive(Debug, Clone)]
pub(crate) struct PyProjectTomlMetadata {
    pub(crate) path: PathBuf,
    pub(crate) has_project_table: bool,
    pub(crate) dependency_groups: BTreeSet<String>,
    pub(crate) optional_dependencies: BTreeSet<String>,
    pub(crate) readme_path: Option<PathBuf>,
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
    #[serde(rename = "optional-dependencies")]
    pub(crate) optional_dependencies: Option<BTreeMap<String, Vec<String>>>,
    pub(crate) readme: Option<ReadmeField>,
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

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum ReadmeField {
    Path(String),
    Table {
        file: Option<String>,
        #[serde(rename = "text")]
        _text: Option<String>,
        #[serde(rename = "content-type")]
        _content_type: Option<String>,
    },
}

pub(crate) fn load_python_project_metadata(
    files: &impl ProjectFiles,
) -> Result<PythonProjectMetadata> {
    let pyproject_path = Path::new("pyproject.toml");
    if !files.exists(pyproject_path) {
        return Ok(PythonProjectMetadata {
            pyproject: PyProjectState::Missing,
        });
    }

    let content = files.read_to_string(pyproject_path)?;
    let raw =
        parse_pyproject_content(&content).context("Unable to parse rendered pyproject.toml")?;
    let dependency_groups = raw
        .dependency_groups
        .as_ref()
        .map(|groups| groups.keys().cloned().collect())
        .unwrap_or_default();
    let optional_dependencies = raw
        .project
        .as_ref()
        .and_then(|project| project.optional_dependencies.as_ref())
        .map(|extras| extras.keys().cloned().collect())
        .unwrap_or_default();
    let readme_path = raw
        .project
        .as_ref()
        .and_then(|project| project.readme.as_ref())
        .and_then(readme_path);

    Ok(PythonProjectMetadata {
        pyproject: PyProjectState::Present(PyProjectTomlMetadata {
            path: pyproject_path.to_path_buf(),
            has_project_table: raw.project.is_some(),
            dependency_groups,
            optional_dependencies,
            readme_path,
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

fn readme_path(readme: &ReadmeField) -> Option<PathBuf> {
    match readme {
        ReadmeField::Path(path) => Some(PathBuf::from(path)),
        ReadmeField::Table { file, .. } => file.as_ref().map(PathBuf::from),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    impl ProjectFiles for PathBuf {
        fn exists(&self, path: &Path) -> bool {
            self.join(path).as_path().exists()
        }

        fn read_to_string(&self, path: &Path) -> Result<String> {
            fs::read_to_string(self.join(path)).map_err(Into::into)
        }
    }

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
    fn extracts_pyproject_metadata() {
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
        let pyproject = metadata.pyproject().unwrap();

        assert!(metadata.has_project_table());
        assert!(pyproject.dependency_groups.contains("dev"));
        assert!(pyproject.optional_dependencies.contains("cli"));
        assert_eq!(pyproject.readme_path, Some(PathBuf::from("README.md")));
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
    fn extracts_table_readme_file() {
        let target = make_temp_dir("metadata-readme-table");
        make_file(
            "pyproject.toml",
            &target,
            r#"
            [project]
            readme = { file = "docs/readme.md", content-type = "text/markdown" }
            "#,
        );

        let metadata = load_python_project_metadata(&target).unwrap();

        assert_eq!(
            metadata.pyproject().unwrap().readme_path,
            Some(PathBuf::from("docs/readme.md"))
        );
    }

    #[test]
    fn ignores_table_readme_text() {
        let target = make_temp_dir("metadata-readme-text");
        make_file(
            "pyproject.toml",
            &target,
            r#"
            [project]
            readme = { text = "inline", content-type = "text/markdown" }
            "#,
        );

        let metadata = load_python_project_metadata(&target).unwrap();

        assert_eq!(metadata.pyproject().unwrap().readme_path, None);
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
