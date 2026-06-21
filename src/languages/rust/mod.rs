use anyhow::Result;
use std::path::PathBuf;

use crate::{Config, languages::LanguageSetupContext};

pub(crate) mod cargo;
pub(crate) mod project;

pub(crate) struct RustSetupContext {
    pub(crate) target_path: PathBuf,
    pub(crate) config: Config,
}

impl From<LanguageSetupContext> for RustSetupContext {
    fn from(ctx: LanguageSetupContext) -> Self {
        Self {
            target_path: ctx.target_path,
            config: ctx.config,
        }
    }
}

pub(crate) trait RustProjectSetup {
    fn setup(&self, ctx: RustSetupContext, plan: project::plan::RustSetupPlan) -> Result<()>;
}
