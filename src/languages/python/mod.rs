use anyhow::Result;
use std::path::PathBuf;

use crate::languages::python::shared::PythonPm;
use crate::{core::config::Config, languages::SetupContext, util::ProjectScope};

use self::shared::{get_python_manager, get_python_project_scope};

pub mod pip;
pub mod shared;
pub mod uv;

pub struct PythonContext {
    pub project_name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub config: Config,
    pub project_scope: ProjectScope,
    pub package_manager: PythonPm,
}

impl TryFrom<SetupContext> for PythonContext {
    type Error = anyhow::Error;

    fn try_from(ctx: SetupContext) -> Result<Self, Self::Error> {
        let project_scope = get_python_project_scope(&ctx.target_path, &ctx.project_name);
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
