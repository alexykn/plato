use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::{ExecutionContext, core::config::TemplateLanguage};

use super::workspace::setup::WorkspaceBuilder;

pub mod setup;

#[derive(Serialize, Debug, Clone)]
pub(crate) struct TemplateContext {
    #[serde(flatten)]
    context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceSetupContext {
    pub(crate) template_context: TemplateContext,
    pub(crate) source_path: PathBuf,
    pub(crate) target_path: PathBuf,
}

impl From<ExecutionContext> for WorkspaceSetupContext {
    fn from(exec_ctx: ExecutionContext) -> Self {
        let template_context = build_template_context(&exec_ctx);
        Self {
            template_context,
            source_path: exec_ctx.source_path,
            target_path: exec_ctx.target_path,
        }
    }
}

fn build_template_context(exec_ctx: &ExecutionContext) -> TemplateContext {
    let mut template_context: HashMap<String, String> = HashMap::new();
    template_context.insert("project_name".to_string(), exec_ctx.project_name.clone());
    match &exec_ctx.source_config.plato.template_language {
        TemplateLanguage::Python => {
            template_context.insert(
                "language_version".to_string(),
                exec_ctx.source_config.python.language_version.clone(),
            );
        }
        TemplateLanguage::Rust => {
            template_context.insert(
                "toolchain".to_string(),
                exec_ctx.source_config.rust.toolchain.clone(),
            );
        }
        TemplateLanguage::Base => {}
    }

    template_context.extend(exec_ctx.source_config.template.context.clone());

    TemplateContext {
        context: template_context,
    }
}

pub(crate) trait WorkspaceSetup {
    fn setup(&self, ctx: WorkspaceSetupContext) -> Result<()>;
}

pub(crate) struct DefaultWorkspaceSetup;

impl WorkspaceSetup for DefaultWorkspaceSetup {
    fn setup(&self, ctx: WorkspaceSetupContext) -> Result<()> {
        create_dir_all(&ctx.target_path)?;
        WorkspaceBuilder::from_source(&ctx.source_path)?
            .render_paths(&ctx.template_context)?
            .render_templates(&ctx.template_context)?
            .flush_to_disk(&ctx.target_path)?;
        Ok(())
    }
}
