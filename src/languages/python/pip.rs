use anyhow::{Result, bail};
use std::path::Path;

use crate::{
    languages::python::{
        PythonContext, PythonPackageManager,
        shared::{dev_groups_from_pyproject, ensure_readme, ensure_requirements},
    },
    util::{ProjectScope, execute_command},
};

pub(crate) struct PipPackageManager;

impl PythonPackageManager for PipPackageManager {
    fn setup(&self, ctx: PythonContext) -> Result<()> {
        let version = ctx.config.plato.language_version;
        let python_command = format!("python{version}");
        let python_venv_args = ["-m", "venv", ".venv"];
        execute_command(&python_command, &python_venv_args, &ctx.target_path)?;

        match ctx.project_scope {
            ProjectScope::Install => self.pip_install_project(&ctx.target_path),
            ProjectScope::Requirements => self.pip_install_requirements(&ctx.target_path),
            ProjectScope::Base => Ok::<(), anyhow::Error>(()),
        }?;
        Ok(())
    }
}

impl PipPackageManager {
    fn pip_install_project(&self, target: &Path) -> Result<()> {
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

    fn pip_install_requirements(&self, target: &Path) -> Result<()> {
        let python_pip_exec = target.join(".venv/bin/python");
        let requirements = match target.join("requirements.txt").exists() {
            true => target.join("requirements.txt"),
            false if ensure_requirements(target)? => target.join(".plato/requirements.txt"),
            false => bail!("Could not find or generate requirements.txt"),
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
