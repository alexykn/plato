use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::ExecutionContext;
use crate::core::config::Config;
use crate::languages::python::pip::PipPackageManagerSetup;
use crate::languages::python::shared::get_python_package_manager;
use crate::languages::rust::RustPackageManagerSetup;
use crate::languages::rust::shared::get_rust_package_manager;
use crate::languages::rust::{
    RustPackageManager, RustSetupContext, cargo::CargoPackageManagerSetup,
};

use crate::languages::python::{
    PythonPackageManager, PythonPackageManagerSetup, PythonSetupContext, uv::UvPackageManagerSetup,
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
        let package_manager = get_python_package_manager(&ctx);
        match package_manager {
            PythonPackageManager::Uv => run_python_setup(&ctx, &UvPackageManagerSetup),
            PythonPackageManager::Pip => run_python_setup(&ctx, &PipPackageManagerSetup),
            PythonPackageManager::None => {
                bail!("Python package manager {package_manager:?} requested but not installed.");
            }
        }?;
        Ok(())
    }
}

pub(crate) struct RustSetup;

impl LanguageSetup for RustSetup {
    fn setup(&self, ctx: LanguageSetupContext) -> Result<()> {
        let package_manager = get_rust_package_manager();
        match package_manager {
            RustPackageManager::Cargo => run_rust_setup(&ctx, &CargoPackageManagerSetup),
            RustPackageManager::None => {
                bail!("Rust package manager {package_manager:?} requested but not installed.")
            }
        }
    }
}

fn run_python_setup<P>(languate_ctx: &LanguageSetupContext, package_manager: &P) -> Result<()>
where
    P: PythonPackageManagerSetup,
{
    let python_ctx = PythonSetupContext::try_from(languate_ctx)?;
    package_manager.setup(python_ctx)?;
    Ok(())
}

fn run_rust_setup<P>(languate_ctx: &LanguageSetupContext, package_manager: &P) -> Result<()>
where
    P: RustPackageManagerSetup,
{
    let rust_ctx = RustSetupContext::try_from(languate_ctx)?;
    package_manager.setup(rust_ctx)?;
    Ok(())
}
