use anyhow::{Result, bail};

use crate::{
    config::{RustProjectScopeConfig, RustProjectTypeConfig},
    languages::{SetupPlan, rust::RustSetupContext},
};

use super::metadata::CargoManifestMetadata;

pub(crate) type RustSetupPlan = SetupPlan<RustSetupMode>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustSetupMode {
    pub(crate) toolchain: RustToolchainSelection,
    pub(crate) steps: Vec<RustSetupStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustToolchainSelection {
    pub(crate) channel: String,
    pub(crate) components: Vec<String>,
    pub(crate) targets: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RustSetupStep {
    EnsureToolchain,
    AddComponents,
    AddTargets,
    CargoInit { project_type: RustProjectType },
    CargoFetch,
    CargoBuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RustProjectType {
    Binary,
    Library,
}

pub(crate) fn resolve_rust_setup_plan(ctx: &RustSetupContext) -> Result<RustSetupPlan> {
    let metadata = CargoManifestMetadata::from_project_dir(&ctx.target_path);
    resolve_rust_setup_plan_with_metadata(ctx, metadata)
}

fn resolve_rust_setup_plan_with_metadata(
    ctx: &RustSetupContext,
    metadata: CargoManifestMetadata,
) -> Result<RustSetupPlan> {
    ensure_cargo_init_supported(ctx, metadata)?;

    let toolchain = RustToolchainSelection {
        channel: ctx.config.rust.toolchain.clone(),
        components: ctx.config.rust.components.clone(),
        targets: ctx.config.rust.targets.clone(),
    };
    let mut steps = Vec::new();

    steps.push(RustSetupStep::EnsureToolchain);
    if !toolchain.components.is_empty() {
        steps.push(RustSetupStep::AddComponents);
    }
    if !toolchain.targets.is_empty() {
        steps.push(RustSetupStep::AddTargets);
    }
    if ctx.config.rust.cargo_init {
        steps.push(RustSetupStep::CargoInit {
            project_type: match ctx.config.rust.project_type {
                RustProjectTypeConfig::Binary => RustProjectType::Binary,
                RustProjectTypeConfig::Library => RustProjectType::Library,
            },
        });
    }
    match ctx.config.rust.project_scope {
        RustProjectScopeConfig::Base => {}
        RustProjectScopeConfig::Fetch => steps.push(RustSetupStep::CargoFetch),
        RustProjectScopeConfig::Build => steps.push(RustSetupStep::CargoBuild),
    }

    Ok(SetupPlan::new(RustSetupMode { toolchain, steps }))
}

fn ensure_cargo_init_supported(
    ctx: &RustSetupContext,
    metadata: CargoManifestMetadata,
) -> Result<()> {
    if !ctx.config.rust.cargo_init || !metadata.exists {
        return Ok(());
    }

    bail!(
        "cannot run cargo init because Cargo.toml already exists; remove rust.cargo_init or remove the manifest"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, languages::LanguageSetupContext};
    use std::path::PathBuf;

    fn ctx() -> RustSetupContext {
        RustSetupContext::from(LanguageSetupContext {
            target_path: PathBuf::from("."),
            config: Config::default(),
        })
    }

    fn resolve(ctx: &RustSetupContext) -> RustSetupPlan {
        resolve_rust_setup_plan_with_metadata(ctx, CargoManifestMetadata { exists: false }).unwrap()
    }

    #[test]
    fn resolves_base_plan_with_toolchain() {
        let mut ctx = ctx();
        ctx.config.rust.toolchain = "nightly".to_string();

        let plan = resolve(&ctx);

        assert_eq!(plan.mode.toolchain.channel, "nightly");
        assert_eq!(plan.mode.steps, vec![RustSetupStep::EnsureToolchain]);
    }

    #[test]
    fn resolves_component_and_target_steps() {
        let mut ctx = ctx();
        ctx.config.rust.components = vec!["rustfmt".to_string(), "clippy".to_string()];
        ctx.config.rust.targets = vec!["wasm32-unknown-unknown".to_string()];

        let plan = resolve(&ctx);

        assert_eq!(
            plan.mode.steps,
            vec![
                RustSetupStep::EnsureToolchain,
                RustSetupStep::AddComponents,
                RustSetupStep::AddTargets,
            ]
        );
    }

    #[test]
    fn resolves_fetch_plan() {
        let mut ctx = ctx();
        ctx.config.rust.project_scope = RustProjectScopeConfig::Fetch;

        let plan = resolve(&ctx);

        assert_eq!(
            plan.mode.steps,
            vec![RustSetupStep::EnsureToolchain, RustSetupStep::CargoFetch]
        );
    }

    #[test]
    fn resolves_build_plan() {
        let mut ctx = ctx();
        ctx.config.rust.project_scope = RustProjectScopeConfig::Build;

        let plan = resolve(&ctx);

        assert_eq!(
            plan.mode.steps,
            vec![RustSetupStep::EnsureToolchain, RustSetupStep::CargoBuild]
        );
    }

    #[test]
    fn resolves_binary_cargo_init_plan() {
        let mut ctx = ctx();
        ctx.config.rust.cargo_init = true;

        let plan = resolve(&ctx);

        assert_eq!(
            plan.mode.steps,
            vec![
                RustSetupStep::EnsureToolchain,
                RustSetupStep::CargoInit {
                    project_type: RustProjectType::Binary,
                },
            ]
        );
    }

    #[test]
    fn resolves_library_cargo_init_plan() {
        let mut ctx = ctx();
        ctx.config.rust.cargo_init = true;
        ctx.config.rust.project_type = RustProjectTypeConfig::Library;

        let plan = resolve(&ctx);

        assert_eq!(
            plan.mode.steps,
            vec![
                RustSetupStep::EnsureToolchain,
                RustSetupStep::CargoInit {
                    project_type: RustProjectType::Library,
                },
            ]
        );
    }

    #[test]
    fn rejects_cargo_init_when_manifest_exists() {
        let mut ctx = ctx();
        ctx.config.rust.cargo_init = true;

        let error =
            resolve_rust_setup_plan_with_metadata(&ctx, CargoManifestMetadata { exists: true })
                .unwrap_err();

        assert!(error.to_string().contains("Cargo.toml already exists"));
    }
}
