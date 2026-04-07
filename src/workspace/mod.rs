use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use crate::core::config::{Config, TemplateLanguage};

use super::workspace::setup::WorkspaceBuilder;

pub mod setup;

#[derive(Serialize)]
struct TemplateContext<'a> {
    #[serde(flatten)]
    base_context: &'a HashMap<&'a str, &'a str>,
    #[serde(flatten)]
    custom_context: &'a HashMap<String, String>,
}

pub(crate) fn setup_base_workspace(
    project_name: &str,
    config: &Config,
    source: &Path,
    target: &Path,
) -> Result<()> {
    let builder = WorkspaceBuilder::scan(source)?;

    let mut base_context: HashMap<&str, &str> = HashMap::new();
    base_context.insert("project_name", project_name);
    match &config.plato.template_language {
        TemplateLanguage::Python => {
            base_context.insert("language_version", &config.python.language_version);
        }
        TemplateLanguage::Rust => {
            base_context.insert("toolchain", &config.rust.toolchain);
        }
        TemplateLanguage::Base => {}
    }

    let template_context = TemplateContext {
        base_context: &base_context,
        custom_context: &config.template.context,
    };

    let mut path_context = config.template.context.clone();
    for (key, value) in &base_context {
        path_context.insert((*key).to_string(), (*value).to_string());
    }

    builder
        .render_paths(&path_context)?
        .render_templates(&template_context)?
        .flush_to_disk(target)?;
    Ok(())
}
