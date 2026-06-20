use anyhow::{Result, bail};

use crate::{
    config::PythonInstallConfig,
    languages::python::{
        PythonPackageManager, PythonPackageManagerSetup, PythonSetupContext,
        project::plan::{PythonInstaller, PythonSetupMode, PythonSetupPlan},
        shared::{editable_install_target, ensure_readme, get_or_create_requirements_file},
    },
    util::execute_command,
};

pub(crate) struct UvPackageManagerSetup;

fn extend_uv_sync_args(args: &mut Vec<String>, install: &PythonInstallConfig) {
    for group in &install.groups {
        args.push("--group".to_string());
        args.push(group.clone());
    }

    for extra in &install.extras {
        args.push("--extra".to_string());
        args.push(extra.clone());
    }
}

impl PythonPackageManagerSetup for UvPackageManagerSetup {
    fn manager(&self) -> PythonPackageManager {
        PythonPackageManager::Uv
    }

    fn setup(&self, ctx: PythonSetupContext, plan: PythonSetupPlan) -> Result<()> {
        execute_command(
            "uv",
            ["venv", "--python", &ctx.config.python.language_version],
            &ctx.target_path,
        )?;

        match plan.mode {
            PythonSetupMode::Base => Ok(()),
            PythonSetupMode::UvSync { install_project } => {
                if install_project {
                    ensure_readme(&ctx.target_path)?;
                }

                let mut sync_args = vec!["sync".to_string()];
                if !install_project {
                    sync_args.push("--no-install-project".to_string());
                }
                extend_uv_sync_args(&mut sync_args, &ctx.config.python.install);
                execute_command("uv", sync_args, &ctx.target_path)
            }
            PythonSetupMode::EditableInstall {
                installer: PythonInstaller::UvPip,
            } => {
                ensure_readme(&ctx.target_path)?;
                let editable_target = editable_install_target(&ctx.config.python.install.extras);
                execute_command(
                    "uv",
                    [
                        "pip".to_string(),
                        "install".to_string(),
                        "-e".to_string(),
                        editable_target,
                    ],
                    &ctx.target_path,
                )
            }
            PythonSetupMode::RequirementsFile {
                installer: PythonInstaller::UvPip,
            } => {
                let requirements_file = get_or_create_requirements_file(&ctx.target_path)?
                    .to_string_lossy()
                    .to_string();
                execute_command(
                    "uv",
                    [
                        "pip".to_string(),
                        "install".to_string(),
                        "-r".to_string(),
                        requirements_file,
                    ],
                    &ctx.target_path,
                )
            }
            _ => bail!("uv setup received a non-uv setup plan: {:?}", plan.mode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install_config(groups: &[&str], extras: &[&str]) -> PythonInstallConfig {
        PythonInstallConfig {
            groups: groups.iter().map(ToString::to_string).collect(),
            extras: extras.iter().map(ToString::to_string).collect(),
        }
    }

    #[test]
    fn uv_sync_args_include_groups_and_extras() {
        let mut args = vec!["sync".to_string()];
        let install = install_config(&["dev", "lint"], &["cli"]);

        extend_uv_sync_args(&mut args, &install);

        assert_eq!(
            args,
            [
                "sync", "--group", "dev", "--group", "lint", "--extra", "cli"
            ]
        );
    }
}
