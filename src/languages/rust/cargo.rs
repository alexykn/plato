use crate::{
    languages::rust::{
        RustProjectSetup, RustSetupContext,
        project::plan::{RustProjectType, RustSetupPlan, RustSetupStep, RustToolchainSelection},
    },
    util::execute_command,
};

pub(crate) struct CargoProjectSetup;

impl RustProjectSetup for CargoProjectSetup {
    fn setup(&self, ctx: RustSetupContext, plan: RustSetupPlan) -> anyhow::Result<()> {
        for step in plan.mode.steps {
            match step {
                RustSetupStep::EnsureToolchain => {
                    ensure_toolchain(&plan.mode.toolchain, &ctx)?;
                }
                RustSetupStep::AddComponents => {
                    add_components(&plan.mode.toolchain, &ctx)?;
                }
                RustSetupStep::AddTargets => {
                    add_targets(&plan.mode.toolchain, &ctx)?;
                }
                RustSetupStep::CargoInit { project_type } => {
                    cargo_init(&plan.mode.toolchain, project_type, &ctx)?;
                }
                RustSetupStep::CargoFetch => {
                    cargo_command(&plan.mode.toolchain, "fetch", &ctx)?;
                }
                RustSetupStep::CargoBuild => {
                    cargo_command(&plan.mode.toolchain, "build", &ctx)?;
                }
            }
        }

        Ok(())
    }
}

fn ensure_toolchain(
    toolchain: &RustToolchainSelection,
    ctx: &RustSetupContext,
) -> anyhow::Result<()> {
    execute_command(
        "rustup",
        ["toolchain", "install", toolchain.channel.as_str()],
        &ctx.target_path,
    )
}

fn add_components(
    toolchain: &RustToolchainSelection,
    ctx: &RustSetupContext,
) -> anyhow::Result<()> {
    let mut args = vec!["component".to_string(), "add".to_string()];
    args.extend(toolchain.components.iter().cloned());
    args.push("--toolchain".to_string());
    args.push(toolchain.channel.clone());

    execute_command("rustup", args, &ctx.target_path)
}

fn add_targets(toolchain: &RustToolchainSelection, ctx: &RustSetupContext) -> anyhow::Result<()> {
    let mut args = vec!["target".to_string(), "add".to_string()];
    args.extend(toolchain.targets.iter().cloned());
    args.push("--toolchain".to_string());
    args.push(toolchain.channel.clone());

    execute_command("rustup", args, &ctx.target_path)
}

fn cargo_init(
    toolchain: &RustToolchainSelection,
    project_type: RustProjectType,
    ctx: &RustSetupContext,
) -> anyhow::Result<()> {
    let project_type_arg = match project_type {
        RustProjectType::Binary => "--bin",
        RustProjectType::Library => "--lib",
    };

    execute_command(
        "cargo",
        [
            format!("+{}", toolchain.channel),
            "init".to_string(),
            project_type_arg.to_string(),
            "--vcs".to_string(),
            "none".to_string(),
        ],
        &ctx.target_path,
    )
}

fn cargo_command(
    toolchain: &RustToolchainSelection,
    command: &str,
    ctx: &RustSetupContext,
) -> anyhow::Result<()> {
    execute_command(
        "cargo",
        [format!("+{}", toolchain.channel), command.to_string()],
        &ctx.target_path,
    )
}
