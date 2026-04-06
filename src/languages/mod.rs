use anyhow::Result;
use std::path::Path;

use crate::core::config::Config;
use python::{
    PythonPackageManager, get_python_manager, get_python_project_scope, setup_pip_project,
    setup_uv_project,
};

pub mod python;
pub mod rust;

pub(crate) fn setup_rust_workspace(
    _project_name: &str,
    _config: &Config,
    _target: &Path,
) -> Result<()> {
    Ok(())
}

pub(crate) fn setup_python_workspace(
    project_name: &str,
    config: &Config,
    target: &Path,
) -> Result<()> {
    let scope = get_python_project_scope(target, project_name);
    match get_python_manager(&config.plato.language_version) {
        PythonPackageManager::Uv => setup_uv_project(&config.plato.language_version, target, scope),
        PythonPackageManager::Pip => {
            setup_pip_project(&config.plato.language_version, target, scope)
        }
        PythonPackageManager::None => {
            eprintln!("No compatible python package manager found");
            Ok(())
        }
    }?;
    Ok(())
}
