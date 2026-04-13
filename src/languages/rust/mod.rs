use crate::Config;
use crate::core::config::{RustProjectScopeConfig, RustProjectTypeConfig};
use crate::languages::LanguageSetupContext;
use crate::languages::rust::shared::{get_rust_project_scope, get_rust_project_type};
use anyhow::Result;
use std::path::PathBuf;

pub mod cargo;
pub mod shared;

#[derive(Debug, Clone, Copy)]
pub enum RustProjectScope {
    Build,
    Fetch,
    Base,
}

#[derive(Debug, Clone, Copy)]
pub enum RustProjectType {
    Binary,
    Library,
}

#[derive(Debug, Clone, Copy)]
pub enum RustPackageManager {
    Cargo,
    None,
}

pub struct RustSetupContext {
    pub project_name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub config: Config,
    pub project_scope: RustProjectScope,
    pub project_type: RustProjectType, // This is cargo specific, there is only cargo for rust so this is fine for now
}

impl TryFrom<LanguageSetupContext> for RustSetupContext {
    type Error = anyhow::Error;
    fn try_from(ctx: LanguageSetupContext) -> Result<Self, Self::Error> {
        let project_scope = match ctx.config.rust.project_scope {
            RustProjectScopeConfig::Auto => get_rust_project_scope(&ctx.target_path)?,
            RustProjectScopeConfig::Base => RustProjectScope::Base,
            RustProjectScopeConfig::Build => RustProjectScope::Build,
            RustProjectScopeConfig::Fetch => RustProjectScope::Fetch,
        };
        let project_type = match ctx.config.rust.project_type {
            RustProjectTypeConfig::Auto => get_rust_project_type(&ctx.target_path)?,
            RustProjectTypeConfig::Binary => RustProjectType::Binary,
            RustProjectTypeConfig::Library => RustProjectType::Library,
        };

        Ok(Self {
            project_name: ctx.project_name,
            source_path: ctx.source_path,
            target_path: ctx.target_path,
            config: ctx.config,
            project_scope,
            project_type,
        })
    }
}

pub(crate) trait RustPackageManagerSetup {
    fn setup(&self, ctx: RustSetupContext) -> Result<()>;
}
