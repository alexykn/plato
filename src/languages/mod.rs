use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::ExecutionContext;
use crate::core::config::Config;
use crate::languages::rust::RustPackageManagerSetup;
use crate::languages::rust::{
    RustPackageManager, RustSetupContext, cargo::CargoPackageManagerSetup,
};

use crate::languages::python::{
    PythonPackageManager, PythonPackageManagerSetup, PythonSetupContext,
    pip::PipPackageManagerSetup, uv::UvPackageManagerSetup,
};

pub mod python;
pub mod rust;

#[derive(Debug, Clone)]
pub struct LanguageSetupContext {
    pub project_name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub config: Config,
}

impl From<&ExecutionContext> for LanguageSetupContext {
    fn from(ctx: &ExecutionContext) -> Self {
        LanguageSetupContext {
            project_name: ctx.project_name.clone(),
            source_path: ctx.source_path.clone(),
            target_path: ctx.target_path.clone(),
            config: ctx.source_config.clone(),
        }
    }
}

pub(crate) trait LanguageSetup {
    fn setup(&self, ctx: LanguageSetupContext) -> Result<()>;
}

pub(crate) struct PythonSetup;

impl LanguageSetup for PythonSetup {
    fn setup(&self, ctx: LanguageSetupContext) -> Result<()> {
        let ctx = PythonSetupContext::try_from(ctx)?;
        match ctx.package_manager {
            PythonPackageManager::Uv => UvPackageManagerSetup.setup(ctx),
            PythonPackageManager::Pip => PipPackageManagerSetup.setup(ctx),
            PythonPackageManager::None => {
                bail!(
                    "Python package manager {:?} requested but not installed.",
                    ctx.package_manager
                );
            }
        }?;
        Ok(())
    }
}

pub(crate) struct RustSetup;

impl LanguageSetup for RustSetup {
    fn setup(&self, ctx: LanguageSetupContext) -> Result<()> {
        let ctx = RustSetupContext::try_from(ctx)?;
        match ctx.package_manager {
            RustPackageManager::Cargo => CargoPackageManagerSetup.setup(ctx),
            RustPackageManager::None => {
                bail!(
                    "Rust package manager {:?} requested but not installed.",
                    ctx.package_manager
                )
            }
        }
    }
}
