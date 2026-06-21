use anyhow::Result;
use std::path::PathBuf;

use crate::ExecutionContext;
use crate::config::Config;
use crate::languages::python::pip::PipPackageManagerSetup;
use crate::languages::python::shared::get_python_package_manager;
use crate::languages::rust::RustProjectSetup;
use crate::languages::rust::{RustSetupContext, cargo::CargoProjectSetup};

use crate::languages::python::{
    PythonPackageManager, PythonPackageManagerSetup, PythonSetupContext, uv::UvPackageManagerSetup,
};

pub(crate) mod python;
pub(crate) mod rust;

#[derive(Debug, Clone)]
pub(crate) struct LanguageSetupContext {
    pub(crate) target_path: PathBuf,
    pub(crate) config: Config,
}

impl From<&ExecutionContext> for LanguageSetupContext {
    fn from(ctx: &ExecutionContext) -> Self {
        LanguageSetupContext {
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
        }?;
        Ok(())
    }
}

pub(crate) struct RustSetup;

impl LanguageSetup for RustSetup {
    fn setup(&self, ctx: LanguageSetupContext) -> Result<()> {
        run_rust_setup(ctx, &CargoProjectSetup)
    }
}

fn run_python_setup<P>(language_ctx: LanguageSetupContext, package_manager: &P) -> Result<()>
where
    P: PythonPackageManagerSetup,
{
    let python_ctx = PythonSetupContext::try_from_language(language_ctx, package_manager.manager());
    let plan = python::project::plan::resolve_python_setup_plan(&python_ctx)?;
    package_manager.setup(python_ctx, plan)?;
    Ok(())
}

fn run_rust_setup<P>(language_ctx: LanguageSetupContext, package_manager: &P) -> Result<()>
where
    P: RustProjectSetup,
{
    let rust_ctx = RustSetupContext::from(language_ctx);
    let plan = rust::project::plan::resolve_rust_setup_plan(&rust_ctx)?;
    package_manager.setup(rust_ctx, plan)?;
    Ok(())
}
