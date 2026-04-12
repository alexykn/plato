use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use crate::core::config::{Config, TemplateLanguage};

use super::workspace::setup::WorkspaceBuilder;

pub mod setup;

#[derive(Serialize)]
struct TemplateContext {
    #[serde(flatten)]
    context: HashMap<String, String>,
}

pub(crate) fn setup_base_workspace(
    project_name: &str,
    config: &Config,
    source: &Path,
    target: &Path,
) -> Result<()> {
    let builder = WorkspaceBuilder::scan(source)?;

    let mut context: HashMap<String, String> = HashMap::new();
    context.insert("project_name".to_string(), project_name.to_string());
    match &config.plato.template_language {
        TemplateLanguage::Python => {
            context.insert(
                "language_version".to_string(),
                config.python.language_version.clone(),
            );
        }
        TemplateLanguage::Rust => {
            context.insert("toolchain".to_string(), config.rust.toolchain.clone());
        }
        TemplateLanguage::Base => {}
    }

    context.extend(config.template.context.clone());

    let template_context = TemplateContext { context };
    builder
        .render_paths(&template_context)?
        .render_templates(&template_context)?
        .flush_to_disk(target)?;
    Ok(())
}
