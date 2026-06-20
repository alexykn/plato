use anyhow::Result;

use crate::{
    languages::python::{
        PythonPackageManagerSetup, PythonProjectScope, PythonSetupContext,
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
                    return execute_command("uv", ["sync"], &ctx.target_path);
                }
                execute_command("uv", ["pip", "install", "-e", "."], &ctx.target_path)
            }
            PythonProjectScope::Requirements => {
                if has_project_table {
                    return execute_command(
                        "uv",
                        ["sync", "--no-install-project"],
                        &ctx.target_path,
                    );
                }
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
