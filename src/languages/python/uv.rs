use anyhow::Result;

use crate::{
    languages::python::{
        PythonPackageManagerSetup, PythonProjectScope, PythonSetupContext, shared::ensure_readme,
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
        match ctx.project_scope {
            PythonProjectScope::Install => {
                ensure_readme(&ctx.target_path)?;
                execute_command("uv", ["sync"], &ctx.target_path)
            }
            PythonProjectScope::Requirements => {
                execute_command("uv", ["sync", "--no-install-project"], &ctx.target_path)
            }
            PythonProjectScope::Base => Ok(()),
        }
    }
}
