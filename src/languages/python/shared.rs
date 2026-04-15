use crate::core::config::PythonPackageManagerConfig;
use crate::languages::LanguageSetupContext;
use crate::languages::python::{PythonPackageManager, PythonProjectScope};
use crate::util::is_installed;
use anyhow::{Context, Ok, Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct PythonVersionedCommands {
    pub(super) requested: String,
    pub(super) major: String,
    pub(super) unknown: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct PyProject {
    project: Option<ProjectTable>,
    // The new standard (PEP 735)
    #[serde(rename = "dependency-groups")]
    dependency_groups: Option<HashMap<String, Vec<String>>>,
    // The specific tool tables
    tool: Option<ToolTable>,
}

#[derive(Deserialize, Debug)]
struct ProjectTable {
    dependencies: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct ToolTable {
    // Handling the legacy uv fields
    uv: Option<UvTable>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct UvTable {
    dev_dependencies: Option<Vec<String>>,
}

pub(crate) fn get_python_project_scope(target: &Path, project_name: &str) -> PythonProjectScope {
    use PythonProjectScope::{Base, Install, Requirements};

    let has_pyproject = target.join("pyproject.toml").exists();
    let has_requirements = target.join("requirements.txt").exists();
    let normalized_project_name = project_name.replace('-', "_");

    let has_src_package = target
        .join(format!("src/{project_name}/__init__.py"))
        .exists()
        || target
            .join(format!("src/{normalized_project_name}/__init__.py"))
            .exists();

    let has_flat_package = target.join(format!("{project_name}/__init__.py")).exists()
        || target
            .join(format!("{normalized_project_name}/__init__.py"))
            .exists();

    if has_pyproject && (has_src_package || has_flat_package) {
        return Install;
    }
    if has_pyproject || has_requirements {
        return Requirements;
    }
    Base
}

pub(crate) fn get_python_package_manager(ctx: &LanguageSetupContext) -> PythonPackageManager {
    match ctx.config.python.package_manager {
        PythonPackageManagerConfig::Auto => resolve_python_package_manager(
            &ctx.config.python.language_version,
            ctx.config.python.pip_config.version_fallback,
        ),
        PythonPackageManagerConfig::Uv => PythonPackageManager::Uv,
        PythonPackageManagerConfig::Pip => PythonPackageManager::Pip,
    }
}

// Keep auto-detection quiet here: pip command fallback warnings are emitted in
// `PipPackageManagerSetup::setup`, so logging them again at detection time would duplicate output.
//
fn resolve_python_package_manager(version: &str, pip_fallback: bool) -> PythonPackageManager {
    if is_installed("uv") {
        return PythonPackageManager::Uv;
    }

    let python_commands = build_python_versioned_commands(version);
    if !pip_fallback && is_installed(&python_commands.requested) {
        return PythonPackageManager::Pip;
    }

    if pip_fallback
        && (is_installed(&python_commands.requested)
            || is_installed(&python_commands.major)
            || is_installed(&python_commands.unknown))
    {
        return PythonPackageManager::Pip;
    }

    eprintln!("No supported python package manager found for 'project_scope: auto'.");
    PythonPackageManager::None
}

pub(crate) fn parse_pyproject(pyproject_path: &Path) -> Result<PyProject> {
    let content = fs::read_to_string(pyproject_path).context(format!(
        "Could not pyproject toml at {}",
        pyproject_path.display()
    ))?;
    let pyrproject: PyProject = toml::from_str(&content)?;
    Ok(pyrproject)
}

pub(crate) fn requirements_from_pyproject(pyproject: PyProject) -> Result<Vec<String>> {
    let mut requirements: Vec<String> = Vec::new();
    if let Some(project_deps) = pyproject.project.and_then(|x| x.dependencies) {
        requirements.extend(project_deps);
    }
    if let Some(group_deps) = pyproject.dependency_groups {
        for (_, dep_list) in group_deps {
            requirements.extend(dep_list);
        }
    }
    if let Some(uv_deps) = pyproject
        .tool
        .and_then(|tool| tool.uv)
        .and_then(|uv| uv.dev_dependencies)
    {
        requirements.extend(uv_deps);
    }
    Ok(requirements)
}

pub(crate) fn dev_groups_from_pyproject(target: &Path) -> Result<Vec<String>> {
    let pyproject_path = target.join("pyproject.toml");
    let pyproject = parse_pyproject(&pyproject_path)?;
    let mut dev_groups: Vec<String> = Vec::new();
    if let Some(group_deps) = pyproject.dependency_groups {
        for (group, _) in group_deps {
            dev_groups.push(group);
        }
    }
    Ok(dev_groups)
}

fn find_file_in_target(target: &Path, file: &str) -> Option<PathBuf> {
    let file_path = target.join(file);
    if file_path.exists() {
        return Some(file_path);
    }
    None
}

fn create_plato_hidden_path(target: &Path) -> Result<PathBuf> {
    let hidden_path = target.join(".plato/");
    fs::create_dir(&hidden_path)?;
    Ok(hidden_path)
}

fn write_requirements_file(target: &Path, requirements: &[String]) -> Result<PathBuf> {
    let file_path = target.join("requirements.txt");
    let content = requirements.join("\n");
    fs::write(&file_path, content)?;
    Ok(file_path)
}

pub(crate) fn get_or_create_requirements_file(target: &Path) -> Result<PathBuf> {
    if let Some(requirements_file) = find_file_in_target(target, "requirements.txt") {
        return Ok(requirements_file);
    }
    if let Some(pyproject_file) = find_file_in_target(target, "pyproject.toml") {
        let pyproject = parse_pyproject(&pyproject_file).context(format!(
            "Unable to parse pyproject.toml at {}",
            pyproject_file.display()
        ))?;
        let requirements = requirements_from_pyproject(pyproject).context(format!(
            "Unable to extract requirements from pyproject.toml at {}",
            pyproject_file.display()
        ))?;
        let hidden_path = create_plato_hidden_path(target).context(format!(
            "Unable to create hidden path at {}/.plato/",
            target.display()
        ))?;
        let requirements_file =
            write_requirements_file(&hidden_path, &requirements).context(format!(
                "Unable to create plato requirements.txt at {}requirements.txt",
                hidden_path.display()
            ))?;
        return Ok(requirements_file);
    }
    bail!("Could not find requirements.txt or pyproject.toml for scope 'requirements' quitting.")
}

pub(crate) fn ensure_readme(target: &Path) -> Result<()> {
    let readme = target.join("README.md");
    if !readme.exists() {
        println!(
            "Generating basic readme at {} to satisfy pip",
            &readme.display()
        );
        fs::write(readme, "# Basic readme generated by Plato")?;
    }
    Ok(())
}

pub(super) fn build_python_versioned_commands(version: &str) -> PythonVersionedCommands {
    let end = version.find('.').unwrap_or(version.len());
    let major_version = &version[..end];
    let python_requested = format!("python{version}");
    let python_major = format!("python{major_version}");
    let python_unknown = String::from("python");

    PythonVersionedCommands {
        requested: python_requested,
        major: python_major,
        unknown: python_unknown,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::create_dir;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    static PYPROJECT_TEST_CONTENT: &str = r"
    [project]
    dependencies = []

    [dependency-groups]
    dev = []
    ";

    fn make_temp_dir(dir_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let temp_path = std::env::temp_dir().join(format!("plato-test-{dir_name}-{unique}"));
        create_dir(&temp_path).expect("Failed to create test directory");
        temp_path
    }

    fn make_file(file: &str, path: &Path, content: &str) {
        let file_path = path.join(file);
        fs::write(&file_path, content)
            .unwrap_or_else(|_| panic!("Failed to create file: {}", file_path.display()));
    }

    #[test]
    fn test_ensure_requirements_requirements_exists() {
        let target = make_temp_dir("ensure_requirements");
        make_file("requirements.txt", &target, "TEST");
        let result = get_or_create_requirements_file(&target).unwrap();

        assert_eq!(result, target.join("requirements.txt"));
    }

    #[test]
    fn test_ensure_requirements_requirements_missing() {
        let target = make_temp_dir("ensure_requirements");
        make_file("pyproject.toml", &target, PYPROJECT_TEST_CONTENT);
        let result = get_or_create_requirements_file(&target).unwrap();

        assert_eq!(result, target.join(".plato/requirements.txt"));
    }
}
