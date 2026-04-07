use anyhow::Result;

use crate::{
    languages::python::{
        PythonContext, PythonPackageManager, PythonProjectScope, shared::ensure_readme,
    },
    util::execute_command,
};

pub(crate) struct UvPackageManager;

impl PythonPackageManager for UvPackageManager {
    fn setup(&self, ctx: PythonContext) -> Result<()> {
        execute_command(
            "uv",
            &["venv", "--python", &ctx.config.plato.language_version],
            &ctx.target_path,
        )?;
        use PythonProjectScope::*;
        match ctx.project_scope {
            Install => {
                ensure_readme(&ctx.target_path)?;
                execute_command("uv", &["sync"], &ctx.target_path)
            }
            Requirements => {
                execute_command("uv", &["sync", "--no-install-project"], &ctx.target_path)
            }
            Base => Ok(()),
        }
    }
}
