use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    ExecutionContext,
    config::PathReplacementConfig,
    config::TemplateLanguage,
    names::{ProjectNameSet, PythonNameSet, RustNameSet},
};

use self::builder::WorkspaceBuilder;
use self::rendered::RenderedWorkspace;

pub(crate) mod builder;
pub(crate) mod content;
pub(crate) mod path_rewrite;
pub(crate) mod rendered;

#[derive(Serialize, Debug, Clone)]
pub(crate) struct TemplateContext {
    #[serde(flatten)]
    context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceRenderContext {
    pub(crate) template_context: TemplateContext,
    pub(crate) path_replacements: BTreeMap<String, PathReplacementConfig>,
    pub(crate) source_path: PathBuf,
}

impl From<&ExecutionContext> for WorkspaceRenderContext {
    fn from(exec_ctx: &ExecutionContext) -> Self {
        Self {
            template_context: build_template_context(exec_ctx),
            path_replacements: exec_ctx.config.path.replace.clone(),
            source_path: exec_ctx.source_path.clone(),
        }
    }
}

fn build_template_context(exec_ctx: &ExecutionContext) -> TemplateContext {
    build_template_context_parts(&exec_ctx.project_name, &exec_ctx.config)
}

pub(crate) fn build_template_context_parts(
    project_name: &str,
    config: &crate::config::Config,
) -> TemplateContext {
    let mut template_context: HashMap<String, String> = HashMap::new();
    let project_names = ProjectNameSet::derive(project_name);

    project_names.insert_context(&mut template_context);
    match &config.plato.template_language {
        TemplateLanguage::Python => {
            PythonNameSet::from_project(&project_names).insert_context(&mut template_context);
            template_context.insert(
                "language_version".to_string(),
                config.python.language_version.clone(),
            );
        }
        TemplateLanguage::Rust => {
            RustNameSet::from_project(&project_names).insert_context(&mut template_context);
            template_context.insert("toolchain".to_string(), config.rust.toolchain.clone());
        }
        TemplateLanguage::Base => {}
    }

    template_context.extend(config.template.context.clone());

    TemplateContext {
        context: template_context,
    }
}

pub(crate) fn render_workspace(ctx: &WorkspaceRenderContext) -> Result<RenderedWorkspace> {
    Ok(WorkspaceBuilder::from_source(&ctx.source_path)?
        .rewrite_paths(&ctx.template_context, &ctx.path_replacements)?
        .render_templates(&ctx.template_context)?
        .build())
}
