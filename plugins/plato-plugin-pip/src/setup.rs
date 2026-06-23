use anyhow::{Result, bail};
use plato_plugin_support::command::run_command;
use std::path::Path;

use crate::config::{PipConfig, PythonScope};
use crate::pyproject::{editable_install_target, ensure_readme, get_or_create_requirements_file};

const RELATIVE_VENV_PYTHON: &str = ".venv/bin/python";

pub(crate) fn setup(workdir: &Path, config: &PipConfig) -> Result<()> {
    validate(config)?;
    let python_command = format!("python{}", config.python);
    run_command(&python_command, ["-m", "venv", ".venv"], workdir)?;

    match config.scope {
        PythonScope::Base => Ok(()),
        PythonScope::Install => install_project(workdir, config),
        PythonScope::Requirements => install_requirements(workdir),
    }
}

fn install_project(workdir: &Path, config: &PipConfig) -> Result<()> {
    ensure_readme(workdir)?;
    let python = workdir
        .join(RELATIVE_VENV_PYTHON)
        .to_string_lossy()
        .to_string();
    let editable_target = editable_install_target(&config.extras);
    run_command(
        &python,
        ["-m", "pip", "install", "-e", editable_target.as_str()],
        workdir,
    )
}

fn install_requirements(workdir: &Path) -> Result<()> {
    let python = workdir
        .join(RELATIVE_VENV_PYTHON)
        .to_string_lossy()
        .to_string();
    let requirements = get_or_create_requirements_file(workdir)?;
    let requirements = requirements.to_string_lossy().to_string();
    run_command(
        &python,
        ["-m", "pip", "install", "-r", requirements.as_str()],
        workdir,
    )
}

fn validate(config: &PipConfig) -> Result<()> {
    match config.scope {
        PythonScope::Install if !config.groups.is_empty() => bail!(
            "pip groups cannot be applied to editable install setup. Remove groups or use a different plugin."
        ),
        PythonScope::Requirements if !config.groups.is_empty() || !config.extras.is_empty() => {
            bail!("pip groups/extras cannot be applied to requirements-file setup.")
        }
        PythonScope::Base if !config.groups.is_empty() || !config.extras.is_empty() => bail!(
            "pip groups/extras require scope = \"install\" with extras only, or remove groups/extras."
        ),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_groups_for_editable_install() {
        let config = PipConfig {
            scope: PythonScope::Install,
            groups: vec!["dev".to_string()],
            ..PipConfig::default()
        };
        assert!(validate(&config).is_err());
    }
}
