use anyhow::Result;
use std::path::PathBuf;

use crate::{RunOptions, core::config::Config};

use crate::languages::python::{
    PythonPackageManager, PythonPackageManagerSetup, PythonSetupContext,
    pip::PipPackageManagerSetup, uv::UvPackageManagerSetup,
};

pub mod python;
pub mod rust;

pub struct SetupContext {
    pub project_name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub config: Config,
}

impl From<&RunOptions> for SetupContext {
    fn from(options: &RunOptions) -> Self {
        SetupContext {
            project_name: options.project_name.clone(),
            source_path: options.source_path.clone(),
            target_path: options.target_path.clone(),
            config: options.config.clone(),
        }
    }
}

pub(crate) trait LanguageSetup {
    fn setup(&self, ctx: SetupContext) -> Result<()>;
}

pub(crate) struct PythonSetup;

impl LanguageSetup for PythonSetup {
    fn setup(&self, ctx: SetupContext) -> Result<()> {
        let ctx = PythonSetupContext::try_from(ctx)?;
        match ctx.package_manager {
            PythonPackageManager::Uv => UvPackageManagerSetup.setup(ctx),
            PythonPackageManager::Pip => PipPackageManagerSetup.setup(ctx),
            PythonPackageManager::None => {
                eprintln!("No compatible python package manager found");
                Ok(())
            }
        }?;
        Ok(())
    }
}

pub(crate) struct RustSetup;

impl LanguageSetup for RustSetup {
    fn setup(&self, _ctx: SetupContext) -> Result<()> {
        Ok(())
    }
}
