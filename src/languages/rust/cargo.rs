use crate::{
    languages::rust::{RustPackageManagerSetup, RustProjectScope, RustProjectType},
    util::execute_command,
};

pub(crate) struct CargoPackageManagerSetup;

impl RustPackageManagerSetup for CargoPackageManagerSetup {
    fn setup(&self, ctx: super::RustSetupContext) -> anyhow::Result<()> {
        if ctx.config.rust.cargo_init {
            match &ctx.project_type {
                RustProjectType::Binary => execute_command(
                    "cargo",
                    &["init", "--bin", "--vcs", "none"],
                    &ctx.target_path,
                ),
                RustProjectType::Library => execute_command(
                    "cargo",
                    &["init", "--lib", "--vcs", "none"],
                    &ctx.target_path,
                ),
            }?;
        }
        match &ctx.project_scope {
            RustProjectScope::Build => execute_command("cargo", &["build"], &ctx.target_path),
            RustProjectScope::Fetch => execute_command("cargo", &["fetch"], &ctx.target_path),
            RustProjectScope::Base => Ok(()),
        }
    }
}
