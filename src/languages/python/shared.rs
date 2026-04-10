use crate::languages::python::{PythonPackageManager, PythonProjectScope};
use crate::util::is_installed;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

// Keep auto-detection quiet here: pip command fallback warnings are emitted in
// `PipPackageManagerSetup::setup`, so logging them again at detection time would duplicate output.
//
pub(crate) fn get_python_package_manager(
    version: &str,
    pip_fallback: bool,
) -> PythonPackageManager {
    if is_installed("uv") {
        return PythonPackageManager::Uv;
    }
    let end = version.find('.').unwrap_or(version.len());
    let major_version = &version[..end];

    if !pip_fallback && is_installed(&format!("python{version}")) {
        return PythonPackageManager::Pip;
    }

    if pip_fallback
        && (is_installed(&format!("python{version}"))
            || is_installed(&format!("python{major_version}"))
            || is_installed("python"))
    {
        return PythonPackageManager::Pip;
    }

    eprintln!("No supported python package manager found for 'project_scope: auto'.");
    PythonPackageManager::None
}

pub(crate) fn parse_pyproject(pyproject_path: &Path) -> Result<PyProject> {
    if !pyproject_path.exists() {
        bail!("This shoud not have happened, how did we get here?!")
    }
    let content = fs::read_to_string(pyproject_path).context(format!(
        "Could not pyproject toml at {}",
        pyproject_path.display()
    ))?;
    let pyrproject: PyProject = toml::from_str(&content)?;
    Ok(pyrproject)
}

pub(crate) fn requirements_from_pyproject(target: &Path) -> Result<Vec<String>> {
    let pyproject_path = target.join("pyproject.toml");
    let pyproject = parse_pyproject(&pyproject_path)?;
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

pub(crate) fn ensure_requirements(target: &Path) -> Result<bool> {
    let req_file = target.join("requirements.txt");
    if !req_file.exists() {
        let plato_hidden_path = target.join(".plato/");
        let plato_req_file = target.join(".plato/requirements.txt");
        let requirements = requirements_from_pyproject(target)?;
        let file_content = requirements.join("\n");
        fs::create_dir_all(plato_hidden_path)?;
        fs::write(&plato_req_file, file_content)?;

        println!(
            "Generating requirements.txt at {} to satisfy pip",
            &plato_req_file.display()
        );
        return Ok(true);
    }
    Ok(false)
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
