use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::{
    ExecutionContext,
    config::PathReplacementConfig,
    config::TemplateLanguage,
    names::{ProjectNameSet, PythonNameSet, RustNameSet},
};

use super::workspace::setup::WorkspaceBuilder;

pub(crate) mod path_rewrite;
pub(crate) mod setup;

#[derive(Serialize, Debug, Clone)]
pub(crate) struct TemplateContext {
    #[serde(flatten)]
    context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceSetupContext {
    pub(crate) template_context: TemplateContext,
    pub(crate) path_replacements: BTreeMap<String, PathReplacementConfig>,
    pub(crate) source_path: PathBuf,
    pub(crate) target_path: PathBuf,
}

impl From<&ExecutionContext> for WorkspaceSetupContext {
    fn from(exec_ctx: &ExecutionContext) -> Self {
        let template_context = build_template_context(exec_ctx);
        Self {
            template_context,
            path_replacements: exec_ctx.config.path.replace.clone(),
            source_path: exec_ctx.source_path.clone(),
            target_path: exec_ctx.target_path.clone(),
        }
    }
}

fn build_template_context(exec_ctx: &ExecutionContext) -> TemplateContext {
    let mut template_context: HashMap<String, String> = HashMap::new();
    let project_names = ProjectNameSet::derive(&exec_ctx.project_name);

    project_names.insert_context(&mut template_context);
    match &exec_ctx.config.plato.template_language {
        TemplateLanguage::Python => {
            PythonNameSet::from_project(&project_names).insert_context(&mut template_context);
            template_context.insert(
                "language_version".to_string(),
                exec_ctx.config.python.language_version.clone(),
            );
        }
        TemplateLanguage::Rust => {
            RustNameSet::from_project(&project_names).insert_context(&mut template_context);
            template_context.insert(
                "toolchain".to_string(),
                exec_ctx.config.rust.toolchain.clone(),
            );
        }
        TemplateLanguage::Base => {}
    }

    template_context.extend(exec_ctx.config.template.context.clone());

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
            .rewrite_paths(&ctx.template_context, &ctx.path_replacements)?
            .render_templates(&ctx.template_context)?
            .flush_to_disk(&ctx.target_path)?;
        Ok(())
    }
}
