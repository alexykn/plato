use anyhow::Result;
use std::path::PathBuf;

use crate::languages::python::shared::PythonPm;
use crate::{core::config::Config, core::config::ProjectScope, languages::SetupContext};

use self::shared::{get_python_manager, get_python_project_scope};

pub mod pip;
pub mod shared;
pub mod uv;

#[derive(Debug, Clone, Copy)]
pub enum PythonProjectScope {
    Requirements,
    Install,
    Base,
}

pub struct PythonContext {
    pub project_name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub config: Config,
    pub project_scope: PythonProjectScope,
    pub package_manager: PythonPm,
}

impl TryFrom<SetupContext> for PythonContext {
    type Error = anyhow::Error;

    fn try_from(ctx: SetupContext) -> Result<Self, Self::Error> {
        use ProjectScope::*;
        let project_scope = match ctx.config.plato.project_scope {
            Auto => get_python_project_scope(&ctx.target_path, &ctx.project_name),
            Base => PythonProjectScope::Base,
            Install => PythonProjectScope::Install,
            Requirements => PythonProjectScope::Requirements,
        };
        let package_manager = get_python_manager(&ctx.config.plato.language_version);

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

pub(crate) trait PythonPackageManager {
    fn setup(&self, ctx: PythonContext) -> Result<()>;
}
