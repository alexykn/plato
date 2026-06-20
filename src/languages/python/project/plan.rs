use anyhow::{Result, bail};

use crate::{
    config::PythonUvSetupConfig,
    languages::{
        SetupPlan,
        python::{PythonPackageManager, PythonProjectScope, PythonSetupContext},
    },
};

pub(crate) type PythonSetupPlan = SetupPlan<PythonSetupMode>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PythonSetupMode {
    Base,
    UvSync { install_project: bool },
    EditableInstall { installer: PythonInstaller },
    RequirementsFile { installer: PythonInstaller },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PythonInstaller {
    UvPip,
    Pip,
}

pub(crate) fn resolve_python_setup_plan(ctx: &PythonSetupContext) -> Result<PythonSetupPlan> {
    ensure_package_manager_config_supported(ctx)?;

    let uv_setup = ctx.config.python.uv_setup();
    let mode = match (ctx.package_manager, ctx.project_scope, uv_setup) {
        (_, PythonProjectScope::Base, _) => PythonSetupMode::Base,
        (PythonPackageManager::Uv, PythonProjectScope::Install, PythonUvSetupConfig::Sync) => {
            PythonSetupMode::UvSync {
                install_project: true,
            }
        }
        (PythonPackageManager::Uv, PythonProjectScope::Install, PythonUvSetupConfig::Editable) => {
            PythonSetupMode::EditableInstall {
                installer: PythonInstaller::UvPip,
            }
        }
        (PythonPackageManager::Uv, PythonProjectScope::Requirements, PythonUvSetupConfig::Sync) => {
            PythonSetupMode::UvSync {
                install_project: false,
            }
        }
        (
            PythonPackageManager::Uv,
            PythonProjectScope::Requirements,
            PythonUvSetupConfig::Editable,
        ) => PythonSetupMode::RequirementsFile {
            installer: PythonInstaller::UvPip,
        },
        (PythonPackageManager::Pip, PythonProjectScope::Install, _) => {
            PythonSetupMode::EditableInstall {
                installer: PythonInstaller::Pip,
            }
        }
        (PythonPackageManager::Pip, PythonProjectScope::Requirements, _) => {
            PythonSetupMode::RequirementsFile {
                installer: PythonInstaller::Pip,
            }
        }
    };

    ensure_install_selectors_supported(ctx, mode)?;

    Ok(SetupPlan::new(mode))
}

fn ensure_package_manager_config_supported(ctx: &PythonSetupContext) -> Result<()> {
    if ctx.package_manager == PythonPackageManager::Uv || ctx.config.python.uv_config.is_none() {
        return Ok(());
    }

    bail!(
        "[python.uv] settings require [python].package_manager = \"uv\". Remove [python.uv] or switch package_manager to \"uv\"."
    )
}

fn ensure_install_selectors_supported(
    ctx: &PythonSetupContext,
    mode: PythonSetupMode,
) -> Result<()> {
    let install = &ctx.config.python.install;
    if install.is_empty() {
        return Ok(());
    }

    match mode {
        PythonSetupMode::UvSync { .. } => Ok(()),
        PythonSetupMode::EditableInstall { .. } if install.groups.is_empty() => Ok(()),
        PythonSetupMode::EditableInstall { .. } => bail!(
            "python.install.groups cannot be applied to editable install setup. Use a pyproject.toml [project] table with [dependency-groups] and uv sync, or remove python.install.groups."
        ),
        PythonSetupMode::RequirementsFile { .. } => bail!(
            "python.install options cannot be applied to requirements-file setup. Remove python.install.groups/extras or use an install setup path that supports them."
        ),
        PythonSetupMode::Base => bail!(
            "python.install options require a Python setup scope that installs dependencies. Set [python].project_scope to \"install\" or \"requirements\", or remove python.install."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, template::UvConfig};
    use std::path::PathBuf;

    fn ctx(
        package_manager: PythonPackageManager,
        project_scope: PythonProjectScope,
    ) -> PythonSetupContext {
        PythonSetupContext {
            target_path: PathBuf::from("."),
            config: Config::default(),
            project_scope,
            package_manager,
        }
    }

    fn ctx_with_install(
        package_manager: PythonPackageManager,
        project_scope: PythonProjectScope,
        groups: &[&str],
        extras: &[&str],
    ) -> PythonSetupContext {
        let mut ctx = ctx(package_manager, project_scope);
        ctx.config.python.install.groups = groups.iter().map(ToString::to_string).collect();
        ctx.config.python.install.extras = extras.iter().map(ToString::to_string).collect();
        ctx
    }

    fn ctx_with_uv_setup(
        project_scope: PythonProjectScope,
        setup: PythonUvSetupConfig,
    ) -> PythonSetupContext {
        let mut ctx = ctx(PythonPackageManager::Uv, project_scope);
        ctx.config.python.uv_config = Some(UvConfig { setup });
        ctx
    }

    #[test]
    fn resolves_uv_install_to_editable_by_default() {
        let plan =
            resolve_python_setup_plan(&ctx(PythonPackageManager::Uv, PythonProjectScope::Install))
                .unwrap();

        assert_eq!(
            plan.mode,
            PythonSetupMode::EditableInstall {
                installer: PythonInstaller::UvPip
            }
        );
    }

    #[test]
    fn resolves_uv_sync_install_when_configured() {
        let plan = resolve_python_setup_plan(&ctx_with_uv_setup(
            PythonProjectScope::Install,
            PythonUvSetupConfig::Sync,
        ))
        .unwrap();

        assert_eq!(
            plan.mode,
            PythonSetupMode::UvSync {
                install_project: true
            }
        );
    }

    #[test]
    fn resolves_uv_sync_requirements_to_no_project_sync() {
        let plan = resolve_python_setup_plan(&ctx_with_uv_setup(
            PythonProjectScope::Requirements,
            PythonUvSetupConfig::Sync,
        ))
        .unwrap();

        assert_eq!(
            plan.mode,
            PythonSetupMode::UvSync {
                install_project: false
            }
        );
    }

    #[test]
    fn resolves_pip_paths() {
        let install =
            resolve_python_setup_plan(&ctx(PythonPackageManager::Pip, PythonProjectScope::Install))
                .unwrap();
        let requirements = resolve_python_setup_plan(&ctx(
            PythonPackageManager::Pip,
            PythonProjectScope::Requirements,
        ))
        .unwrap();

        assert_eq!(
            install.mode,
            PythonSetupMode::EditableInstall {
                installer: PythonInstaller::Pip
            }
        );
        assert_eq!(
            requirements.mode,
            PythonSetupMode::RequirementsFile {
                installer: PythonInstaller::Pip
            }
        );
    }

    #[test]
    fn resolves_base_for_every_manager() {
        let plan =
            resolve_python_setup_plan(&ctx(PythonPackageManager::Pip, PythonProjectScope::Base))
                .unwrap();

        assert_eq!(plan.mode, PythonSetupMode::Base);
    }

    #[test]
    fn rejects_groups_for_editable_install() {
        let error = resolve_python_setup_plan(&ctx_with_install(
            PythonPackageManager::Pip,
            PythonProjectScope::Install,
            &["dev"],
            &[],
        ))
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("python.install.groups cannot be applied")
        );
    }

    #[test]
    fn rejects_install_options_for_requirements_file() {
        let error = resolve_python_setup_plan(&ctx_with_install(
            PythonPackageManager::Pip,
            PythonProjectScope::Requirements,
            &[],
            &["cli"],
        ))
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("python.install options cannot be applied")
        );
    }

    #[test]
    fn rejects_uv_settings_for_pip() {
        let mut ctx = ctx(PythonPackageManager::Pip, PythonProjectScope::Install);
        ctx.config.python.uv_config = Some(UvConfig {
            setup: PythonUvSetupConfig::Sync,
        });

        let error = resolve_python_setup_plan(&ctx).unwrap_err();

        assert!(error.to_string().contains("[python.uv] settings require"));
    }
}
