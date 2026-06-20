use anyhow::Result;
use std::path::PathBuf;

use crate::{
    config::{Config, PythonProjectScopeConfig},
    languages::LanguageSetupContext,
};

pub(crate) mod pip;
pub(crate) mod project;
pub(crate) mod shared;
pub(crate) mod uv;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PythonProjectScope {
    Requirements,
    Install,
    Base,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PythonPackageManager {
    Pip,
    Uv,
}

pub(crate) struct PythonSetupContext {
    pub(crate) target_path: PathBuf,
    pub(crate) config: Config,
    pub(crate) project_scope: PythonProjectScope,
    pub(crate) package_manager: PythonPackageManager,
}

impl PythonSetupContext {
    pub(crate) fn try_from_language(
        ctx: LanguageSetupContext,
        package_manager: PythonPackageManager,
    ) -> Self {
        let project_scope = match ctx.config.python.project_scope {
            PythonProjectScopeConfig::Base => PythonProjectScope::Base,
            PythonProjectScopeConfig::Install => PythonProjectScope::Install,
            PythonProjectScopeConfig::Requirements => PythonProjectScope::Requirements,
        };

        Self {
            target_path: ctx.target_path,
            config: ctx.config,
            project_scope,
            package_manager,
        }
    }
}

pub(crate) trait PythonPackageManagerSetup {
    fn manager(&self) -> PythonPackageManager;

    fn setup(&self, ctx: PythonSetupContext, plan: project::plan::PythonSetupPlan) -> Result<()>;
}
