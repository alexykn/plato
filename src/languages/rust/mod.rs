use crate::{
    Config,
    config::{RustProjectScopeConfig, RustProjectTypeConfig},
    languages::LanguageSetupContext,
};
use anyhow::Result;
use std::path::PathBuf;

pub(crate) mod cargo;

#[derive(Debug, Clone, Copy)]
pub(crate) enum RustProjectScope {
    Build,
    Fetch,
    Base,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RustProjectType {
    Binary,
    Library,
}

pub(crate) struct RustSetupContext {
    pub(crate) target_path: PathBuf,
    pub(crate) config: Config,
    pub(crate) project_scope: RustProjectScope,
    pub(crate) project_type: RustProjectType, // This is cargo specific, there is only cargo for rust so this is fine for now
}

impl From<LanguageSetupContext> for RustSetupContext {
    fn from(ctx: LanguageSetupContext) -> Self {
        let project_scope = match ctx.config.rust.project_scope {
            RustProjectScopeConfig::Base => RustProjectScope::Base,
            RustProjectScopeConfig::Build => RustProjectScope::Build,
            RustProjectScopeConfig::Fetch => RustProjectScope::Fetch,
        };
        let project_type = match ctx.config.rust.project_type {
            RustProjectTypeConfig::Binary => RustProjectType::Binary,
            RustProjectTypeConfig::Library => RustProjectType::Library,
        };

        Self {
            target_path: ctx.target_path,
            config: ctx.config,
            project_scope,
            project_type,
        }
    }
}

pub(crate) trait RustPackageManagerSetup {
    fn setup(&self, ctx: RustSetupContext) -> Result<()>;
}
