use anyhow::Result;

use crate::{
    languages::python::{
        PythonPackageManagerSetup, PythonProjectScope, PythonSetupContext,
        install::{
            editable_install_target, ensure_no_groups_for_editable_install,
            ensure_no_install_options_for_requirements_file, extend_uv_sync_args,
        },
        shared::{ensure_readme, get_or_create_requirements_file, has_pyproject_project_table},
    },
    util::execute_command,
};

pub(crate) struct UvPackageManagerSetup;

impl PythonPackageManagerSetup for UvPackageManagerSetup {
    fn setup(&self, ctx: PythonSetupContext) -> Result<()> {
        execute_command(
            "uv",
            ["venv", "--python", &ctx.config.python.language_version],
            &ctx.target_path,
        )?;
        let has_project_table = has_pyproject_project_table(&ctx.target_path)?;
        match ctx.project_scope {
            PythonProjectScope::Install => {
                ensure_readme(&ctx.target_path)?;
                if has_project_table {
                    let mut sync_args = vec!["sync".to_string()];
                    extend_uv_sync_args(&mut sync_args, &ctx.config.python.install);
                    return execute_command("uv", sync_args, &ctx.target_path);
                }
                ensure_no_groups_for_editable_install(&ctx.config.python.install)?;
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
            PythonProjectScope::Requirements => {
                if has_project_table {
                    let mut sync_args = ["sync".to_string(), "--no-install-project".to_string()]
                        .into_iter()
                        .collect();
                    extend_uv_sync_args(&mut sync_args, &ctx.config.python.install);
                    return execute_command("uv", sync_args, &ctx.target_path);
                }
                ensure_no_install_options_for_requirements_file(&ctx.config.python.install)?;
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
            PythonProjectScope::Base => Ok(()),
        }
    }
}
