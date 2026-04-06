use anyhow::Result;

use crate::{
    languages::python::{PythonContext, PythonPackageManager, shared::ensure_readme},
    util::{ProjectScope, execute_command},
};

pub(crate) struct UvPackageManager;

impl PythonPackageManager for UvPackageManager {
    fn setup(&self, ctx: PythonContext) -> Result<()> {
        execute_command(
            "uv",
            &["venv", "--python", &ctx.config.plato.language_version],
            &ctx.target_path,
        )?;
        match ctx.project_scope {
            ProjectScope::Install => {
                ensure_readme(&ctx.target_path)?;
                execute_command("uv", &["sync"], &ctx.target_path)
            }
            ProjectScope::Requirements => {
                execute_command("uv", &["sync", "--no-install-project"], &ctx.target_path)
            }
            ProjectScope::Base => Ok(()),
        }
    }
}
