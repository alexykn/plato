use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::config::{Config, TemplateLanguage};

use super::workspace::setup::{FileContent, build_target_map, flush_to_disk, scan_source_map};

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
) -> Result<HashMap<PathBuf, FileContent>> {
    let source_map = scan_source_map(source)?;

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

    let target_map = build_target_map(source_map, &base_context, &template_context)?;
    flush_to_disk(&target_map, target)?;
    Ok(target_map)
}
