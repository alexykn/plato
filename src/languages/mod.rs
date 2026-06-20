use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::ExecutionContext;
use crate::config::Config;
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
use crate::validation::files::FilesystemProjectFiles;

pub(crate) mod python;
pub(crate) mod rust;

#[derive(Debug, Clone)]
pub(crate) struct LanguageSetupContext {
    pub(crate) project_name: String,
    pub(crate) target_path: PathBuf,
    pub(crate) config: Config,
}

impl From<&ExecutionContext> for LanguageSetupContext {
    fn from(ctx: &ExecutionContext) -> Self {
        LanguageSetupContext {
            project_name: ctx.project_name.clone(),
            target_path: ctx.target_path.clone(),
            config: ctx.config.clone(),
        }
    }
}

pub(crate) trait LanguageSetup {
    fn setup(&self, ctx: LanguageSetupContext) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SetupPlan<M> {
    pub(crate) mode: M,
}

impl<M> SetupPlan<M> {
    pub(crate) const fn new(mode: M) -> Self {
        Self { mode }
    }
}

pub(crate) struct PythonSetup;

impl LanguageSetup for PythonSetup {
    fn setup(&self, ctx: LanguageSetupContext) -> Result<()> {
        let package_manager = get_python_package_manager(&ctx);
        match package_manager {
            PythonPackageManager::Uv => run_python_setup(ctx, &UvPackageManagerSetup),
            PythonPackageManager::Pip => run_python_setup(ctx, &PipPackageManagerSetup),
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
            RustPackageManager::Cargo => run_rust_setup(ctx, &CargoPackageManagerSetup),
            RustPackageManager::None => {
                bail!("Rust package manager {package_manager:?} requested but not installed.")
            }
        }
    }
}

fn run_python_setup<P>(language_ctx: LanguageSetupContext, package_manager: &P) -> Result<()>
where
    P: PythonPackageManagerSetup,
{
    let python_ctx = PythonSetupContext::try_from_language(language_ctx, package_manager.manager());
    let files = FilesystemProjectFiles::new(python_ctx.target_path.clone());
    let metadata = python::project::metadata::load_python_project_metadata(&files)?;
    let plan = python::project::plan::resolve_python_setup_plan(&python_ctx, &metadata)?;
    package_manager.setup(python_ctx, plan)?;
    Ok(())
}

fn run_rust_setup<P>(language_ctx: LanguageSetupContext, package_manager: &P) -> Result<()>
where
    P: RustPackageManagerSetup,
{
    let rust_ctx = RustSetupContext::try_from(language_ctx)?;
    package_manager.setup(rust_ctx)?;
    Ok(())
}
