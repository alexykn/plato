use anyhow::Result;
use std::path::PathBuf;

use crate::{
    core::config::{Config, PythonProjectScopeConfig},
    languages::LanguageSetupContext,
};

use self::shared::get_python_project_scope;

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

pub(crate) struct PythonSetupContext {
    pub(crate) target_path: PathBuf,
    pub(crate) config: Config,
    pub(crate) project_scope: PythonProjectScope,
}

impl TryFrom<&LanguageSetupContext> for PythonSetupContext {
    type Error = anyhow::Error;

    fn try_from(ctx: &LanguageSetupContext) -> Result<Self, Self::Error> {
        let project_scope = match ctx.config.python.project_scope {
            PythonProjectScopeConfig::Auto => {
                get_python_project_scope(&ctx.target_path, &ctx.project_name)
            }
            PythonProjectScopeConfig::Base => PythonProjectScope::Base,
            PythonProjectScopeConfig::Install => PythonProjectScope::Install,
            PythonProjectScopeConfig::Requirements => PythonProjectScope::Requirements,
        };

        Ok(Self {
            target_path: ctx.target_path.clone(),
            config: ctx.config.clone(),
            project_scope,
        })
    }
}

pub(crate) trait PythonPackageManagerSetup {
    fn setup(&self, ctx: PythonSetupContext) -> Result<()>;
}
