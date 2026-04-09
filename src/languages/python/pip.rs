use anyhow::{Context, Result};
use std::path::Path;

use crate::{
    languages::python::{
        PythonPackageManagerSetup, PythonProjectScope, PythonSetupContext,
        shared::{dev_groups_from_pyproject, ensure_readme, ensure_requirements},
    },
    util::execute_command,
};

pub(crate) struct PipPackageManagerSetup;

impl PythonPackageManagerSetup for PipPackageManagerSetup {
    fn setup(&self, ctx: PythonSetupContext) -> Result<()> {
        let version = ctx.config.python.language_version;
        let python_requested = format!("python{version}");
        let python_major = format!("python{}", version.split('.').next().unwrap());
        let python_unknown = "python";

        let python_venv_args = ["-m", "venv", ".venv"];
        execute_command(&python_requested, &python_venv_args, &ctx.target_path)
            .inspect_err(|_| {
                eprint!(
                    "WARNING: Unable to execute {python_requested}, falling back to {python_major}"
                );
            })
            .or_else(|_| execute_command(&python_major, &python_venv_args, &ctx.target_path))
            .inspect_err(|_| {
                eprintln!(
                    "WARNING: Unable to execute {python_major}, falling back to {python_unknown}"
                );
            })
            .or_else(|_| execute_command(python_unknown, &python_venv_args, &ctx.target_path))
            .with_context(|| format!("Unable to execute {python_unknown}"))?;

        match ctx.project_scope {
            PythonProjectScope::Install => Self::pip_install_project(&ctx.target_path),
            PythonProjectScope::Requirements => Self::pip_install_requirements(&ctx.target_path),
            PythonProjectScope::Base => Ok::<(), anyhow::Error>(()),
        }?;
        Ok(())
    }
}

impl PipPackageManagerSetup {
    fn pip_install_project(target: &Path) -> Result<()> {
        let python_pip_exec = target.join(".venv/bin/python");
        let mut python_pip_args = vec!["-m", "pip", "install", "-e", "."];
        let dev_groups = dev_groups_from_pyproject(target)?;

        for group in &dev_groups {
            python_pip_args.push("--group");
            python_pip_args.push(group);
        }

        ensure_readme(target)?;
        execute_command(&python_pip_exec.to_string_lossy(), &python_pip_args, target)?;
        Ok(())
    }

    fn pip_install_requirements(target: &Path) -> Result<()> {
        let python_pip_exec = target.join(".venv/bin/python");
        let requirements = match target.join("requirements.txt").exists() {
            true => target.join("requirements.txt"),
            false if ensure_requirements(target)? => target.join(".plato/requirements.txt"),
            false => {
                println!("WARNING: Could not find or generate requirements.txt");
                return Ok(());
            }
        };
        let python_pip_args = [
            "-m",
            "pip",
            "install",
            "-r",
            &requirements.to_string_lossy(),
        ];
        execute_command(&python_pip_exec.to_string_lossy(), &python_pip_args, target)?;
        Ok(())
    }
}
