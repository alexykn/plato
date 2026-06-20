use anyhow::{Result, bail};

use crate::languages::{
    SetupPlan,
    python::{PythonPackageManager, PythonProjectScope, PythonSetupContext},
};

use super::metadata::PythonProjectMetadata;

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

pub(crate) fn resolve_python_setup_plan(
    ctx: &PythonSetupContext,
    metadata: &PythonProjectMetadata,
) -> Result<PythonSetupPlan> {
    let has_project_table = metadata.has_project_table();
    let mode = match (ctx.package_manager, ctx.project_scope) {
        (_, PythonProjectScope::Base) => PythonSetupMode::Base,
        (PythonPackageManager::Uv, PythonProjectScope::Install) if has_project_table => {
            PythonSetupMode::UvSync {
                install_project: true,
            }
        }
        (PythonPackageManager::Uv, PythonProjectScope::Install) => {
            PythonSetupMode::EditableInstall {
                installer: PythonInstaller::UvPip,
            }
        }
        (PythonPackageManager::Uv, PythonProjectScope::Requirements) if has_project_table => {
            PythonSetupMode::UvSync {
                install_project: false,
            }
        }
        (PythonPackageManager::Uv, PythonProjectScope::Requirements) => {
            PythonSetupMode::RequirementsFile {
                installer: PythonInstaller::UvPip,
            }
        }
        (PythonPackageManager::Pip, PythonProjectScope::Install) => {
            PythonSetupMode::EditableInstall {
                installer: PythonInstaller::Pip,
            }
        }
        (PythonPackageManager::Pip, PythonProjectScope::Requirements) => {
            PythonSetupMode::RequirementsFile {
                installer: PythonInstaller::Pip,
            }
        }
        (PythonPackageManager::None, _) => {
            bail!(
                "Python package manager {:?} requested but not installed.",
                ctx.package_manager
            )
        }
    };

    Ok(SetupPlan::new(mode))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        languages::python::project::metadata::{PyProjectState, PyProjectTomlMetadata},
    };
    use std::{collections::BTreeSet, path::PathBuf};

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

    fn metadata(has_project_table: bool) -> PythonProjectMetadata {
        if !has_project_table {
            return PythonProjectMetadata {
                pyproject: PyProjectState::Missing,
            };
        }

        PythonProjectMetadata {
            pyproject: PyProjectState::Present(PyProjectTomlMetadata {
                path: PathBuf::from("pyproject.toml"),
                has_project_table: true,
                dependency_groups: BTreeSet::new(),
                optional_dependencies: BTreeSet::new(),
                readme_path: None,
            }),
        }
    }

    #[test]
    fn resolves_uv_install_with_project_table_to_sync() {
        let plan = resolve_python_setup_plan(
            &ctx(PythonPackageManager::Uv, PythonProjectScope::Install),
            &metadata(true),
        )
        .unwrap();

        assert_eq!(
            plan.mode,
            PythonSetupMode::UvSync {
                install_project: true
            }
        );
    }

    #[test]
    fn resolves_uv_requirements_with_project_table_to_no_project_sync() {
        let plan = resolve_python_setup_plan(
            &ctx(PythonPackageManager::Uv, PythonProjectScope::Requirements),
            &metadata(true),
        )
        .unwrap();

        assert_eq!(
            plan.mode,
            PythonSetupMode::UvSync {
                install_project: false
            }
        );
    }

    #[test]
    fn resolves_legacy_uv_paths() {
        let install = resolve_python_setup_plan(
            &ctx(PythonPackageManager::Uv, PythonProjectScope::Install),
            &metadata(false),
        )
        .unwrap();
        let requirements = resolve_python_setup_plan(
            &ctx(PythonPackageManager::Uv, PythonProjectScope::Requirements),
            &metadata(false),
        )
        .unwrap();

        assert_eq!(
            install.mode,
            PythonSetupMode::EditableInstall {
                installer: PythonInstaller::UvPip
            }
        );
        assert_eq!(
            requirements.mode,
            PythonSetupMode::RequirementsFile {
                installer: PythonInstaller::UvPip
            }
        );
    }

    #[test]
    fn resolves_pip_paths() {
        let install = resolve_python_setup_plan(
            &ctx(PythonPackageManager::Pip, PythonProjectScope::Install),
            &metadata(true),
        )
        .unwrap();
        let requirements = resolve_python_setup_plan(
            &ctx(PythonPackageManager::Pip, PythonProjectScope::Requirements),
            &metadata(true),
        )
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
        let plan = resolve_python_setup_plan(
            &ctx(PythonPackageManager::Pip, PythonProjectScope::Base),
            &metadata(true),
        )
        .unwrap();

        assert_eq!(plan.mode, PythonSetupMode::Base);
    }
}
