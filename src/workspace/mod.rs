use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::config::Config;

use super::workspace::setup::{FileContent, build_target_map, flush_to_disk, scan_source_map};

pub mod setup;

#[derive(Serialize)]
struct TemplateContext<'a> {
    project_name: &'a str,
    language_version: &'a str,

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

    let mut path_context = HashMap::new();
    path_context.insert("project_name", project_name);
    path_context.insert("project_version", &config.plato.language_version);

    let template_context = TemplateContext {
        project_name,
        language_version: &config.plato.language_version,
        custom_context: &config.template.context,
    };

    let target_map = build_target_map(source_map, &path_context, &template_context)?;
    flush_to_disk(&target_map, target)?;
    Ok(target_map)
}
