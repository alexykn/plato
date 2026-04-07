use anyhow::Result;
use std::path::PathBuf;

use crate::{
    core::config::Config,
    core::config::{PythonPackageManagerConfig, PythonProjectScopeConfig},
    languages::SetupContext,
};

use self::shared::{get_python_package_manager, get_python_project_scope};

pub mod pip;
pub mod shared;
pub mod uv;

#[derive(Debug, Clone, Copy)]
pub enum PythonProjectScope {
    Requirements,
    Install,
    Base,
}

#[derive(Debug, Clone, Copy)]
pub enum PythonPackageManager {
    Pip,
    Uv,
    None,
}

pub struct PythonSetupContext {
    pub project_name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub config: Config,
    pub project_scope: PythonProjectScope,
    pub package_manager: PythonPackageManager,
}

impl TryFrom<SetupContext> for PythonSetupContext {
    type Error = anyhow::Error;

    fn try_from(ctx: SetupContext) -> Result<Self, Self::Error> {
        let project_scope = match ctx.config.python.project_scope {
            PythonProjectScopeConfig::Auto => {
                get_python_project_scope(&ctx.target_path, &ctx.project_name)
            }
            PythonProjectScopeConfig::Base => PythonProjectScope::Base,
            PythonProjectScopeConfig::Install => PythonProjectScope::Install,
            PythonProjectScopeConfig::Requirements => PythonProjectScope::Requirements,
        };
        let package_manager = match ctx.config.python.package_manager {
            PythonPackageManagerConfig::Auto => {
                get_python_package_manager(&ctx.config.python.language_version)
            }
            PythonPackageManagerConfig::Uv => PythonPackageManager::Uv,
            PythonPackageManagerConfig::Pip => PythonPackageManager::Pip,
        };

        Ok(Self {
            project_name: ctx.project_name,
            source_path: ctx.source_path,
            target_path: ctx.target_path,
            config: ctx.config,
            project_scope,
            package_manager,
        })
    }
}

pub(crate) trait PythonPackageManagerSetup {
    fn setup(&self, ctx: PythonSetupContext) -> Result<()>;
}
