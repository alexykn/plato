use anyhow::Result;

use crate::config::{Config, TemplateLanguage};
use crate::languages::python::project::validation::validate_python_project;
use crate::validation::report::ValidationReport;
use crate::workspace::rendered::RenderedWorkspace;

#[derive(Debug, Clone)]
pub(crate) struct ValidationProjectContext {
    pub(crate) project_name: String,
    pub(crate) config: Config,
}

pub(crate) fn validate_rendered_project(
    ctx: &ValidationProjectContext,
    files: &RenderedWorkspace,
) -> Result<ValidationReport> {
    match ctx.config.plato.template_language {
        TemplateLanguage::Python => validate_python_project(ctx, files),
        TemplateLanguage::Rust | TemplateLanguage::Base => Ok(ValidationReport::default()),
    }
}
