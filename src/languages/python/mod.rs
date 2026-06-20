use anyhow::Result;
use std::path::PathBuf;

use crate::{
    config::{Config, PythonProjectScopeConfig},
    languages::LanguageSetupContext,
};

use self::{install::ensure_supported_scope, shared::get_python_project_scope};

pub(crate) mod install;
pub(crate) mod pip;
pub(crate) mod shared;
pub(crate) mod uv;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PythonProjectScope {
    Requirements,
    Install,
    Base,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PythonPackageManager {
    Pip,
    Uv,
    None,
}

pub(crate) struct PythonSetupContext {
    pub(crate) target_path: PathBuf,
    pub(crate) config: Config,
    pub(crate) project_scope: PythonProjectScope,
}

impl TryFrom<LanguageSetupContext> for PythonSetupContext {
    type Error = anyhow::Error;

    fn try_from(ctx: LanguageSetupContext) -> Result<Self, Self::Error> {
        let project_scope = match ctx.config.python.project_scope {
            PythonProjectScopeConfig::Auto => {
                get_python_project_scope(&ctx.target_path, &ctx.project_name)
            }
            PythonProjectScopeConfig::Base => PythonProjectScope::Base,
            PythonProjectScopeConfig::Install => PythonProjectScope::Install,
            PythonProjectScopeConfig::Requirements => PythonProjectScope::Requirements,
        };

        ensure_supported_scope(project_scope, &ctx.config.python.install)?;

        Ok(Self {
            target_path: ctx.target_path,
            config: ctx.config,
            project_scope,
        })
    }
}

pub(crate) trait PythonPackageManagerSetup {
    fn setup(&self, ctx: PythonSetupContext) -> Result<()>;
}
