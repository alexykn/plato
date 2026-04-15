use super::shared::{build_python_versioned_commands, get_or_create_requirements_file};
use anyhow::{Context, Result};
use std::path::Path;

static PYTHON_VENV_CREATION_ARGS: [&str; 3] = ["-m", "venv", ".venv"];
static RELATIVE_VENV_PATH: &str = ".venv/bin/python";

// This can't be static because we gotta extend it later
//
fn get_pip_install_args() -> Vec<String> {
    ["-m", "pip", "install"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

use crate::{
    languages::python::{
        PythonPackageManagerSetup, PythonProjectScope, PythonSetupContext,
        shared::{dev_groups_from_pyproject, ensure_readme},
    },
    util::execute_command,
};

pub(crate) struct PipPackageManagerSetup;

impl PythonPackageManagerSetup for PipPackageManagerSetup {
    fn setup(&self, ctx: PythonSetupContext) -> Result<()> {
        if ctx.config.python.pip_config.version_fallback {
            Self::setup_venv_with_fallback(&ctx.config.python.language_version, &ctx.target_path)?;
        } else {
            Self::setup_venv(&ctx.config.python.language_version, &ctx.target_path)?;
        }

        match ctx.project_scope {
            PythonProjectScope::Install => Self::install_project(&ctx.target_path),
            PythonProjectScope::Requirements => Self::install_requirements(&ctx.target_path),
            PythonProjectScope::Base => Ok::<(), anyhow::Error>(()),
        }?;
        Ok(())
    }
}

impl PipPackageManagerSetup {
    fn setup_venv(version: &str, target: &Path) -> Result<()> {
        let python_command = format!("python{version}");
        execute_command(&python_command, PYTHON_VENV_CREATION_ARGS, target)?;
        Ok(())
    }

    fn setup_venv_with_fallback(version: &str, target: &Path) -> Result<()> {
        let python_commands = build_python_versioned_commands(version);

        execute_command(
            &python_commands.requested,
            PYTHON_VENV_CREATION_ARGS,
            target,
        )
        .inspect_err(|e| {
            eprintln!(
                "{e}\nWARNING: Unable to execute {}, falling back to {}",
                python_commands.requested, python_commands.major
            );
        })
        .or_else(|_| execute_command(&python_commands.major, PYTHON_VENV_CREATION_ARGS, target))
        .inspect_err(|e| {
            eprintln!(
                "{e}\nWARNING: Unable to execute {}, falling back to {}",
                python_commands.major, python_commands.unknown
            );
        })
        .or_else(|_| execute_command(&python_commands.unknown, PYTHON_VENV_CREATION_ARGS, target))
        .with_context(|| {
            format!(
                "Unable to create venv with any Python command: {}, {}, {}",
                python_commands.requested, python_commands.major, python_commands.unknown
            )
        })?;
        Ok(())
    }

    fn install_project(target: &Path) -> Result<()> {
        let python_pip_exec = target.join(RELATIVE_VENV_PATH);
        let dev_groups = dev_groups_from_pyproject(target)?;
        let mut pip_install_args = get_pip_install_args();

        pip_install_args.extend(["-e".to_string(), ".".to_string()]);
        for group in &dev_groups {
            pip_install_args.push("--group".to_string());
            pip_install_args.push(group.clone());
        }
        ensure_readme(target)?;
        execute_command(
            &python_pip_exec.to_string_lossy(),
            &pip_install_args,
            target,
        )?;
        Ok(())
    }

    fn install_requirements(target: &Path) -> Result<()> {
        let python_pip_exec = target.join(RELATIVE_VENV_PATH);
        let requirements_file = get_or_create_requirements_file(target)?
            .to_string_lossy()
            .to_string();
        let mut pip_install_args = get_pip_install_args();
        pip_install_args.extend(["-r".to_string(), requirements_file]);

        execute_command(
            &python_pip_exec.to_string_lossy(),
            &pip_install_args,
            target,
        )?;
        Ok(())
    }
}
