use anyhow::Result;
use std::{fs::create_dir_all, path::PathBuf};

pub mod core;
pub mod languages;
pub mod util;
pub mod workspace;

use crate::core::config::{Config, TemplateLanguage};
use crate::core::guard::ProjectGuard;
use crate::languages::{LanguageSetup, PythonSetup, RustSetup, SetupContext};
use crate::util::setup_git;
use crate::workspace::setup_base_workspace;

pub struct RunOptions {
    pub template_name: String,
    pub project_name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub config: Config,
}

/// Run the CLI.
///
/// # Errors
/// Returns an error if argument parsing, template loading, filesystem access,
/// template rendering, or project setup fails.
pub fn run(options: &RunOptions) -> Result<()> {
    let mut guard = ProjectGuard::new(options.target_path.clone());
    let should_setup_git: bool = options.config.plato.setup_git;
    let ctx = SetupContext::from(options);
    create_dir_all(&options.target_path)?;
    setup_base_workspace(
        &options.project_name,
        &options.config,
        &options.source_path,
        &options.target_path,
    )?;

    match &options.config.plato.template_language {
        TemplateLanguage::Python => PythonSetup.setup(ctx),
        TemplateLanguage::Rust => RustSetup.setup(ctx),
        TemplateLanguage::Base => Ok(()),
    }?;
    if should_setup_git {
        setup_git(&options.target_path)?;
    }
    guard.release();
    Ok(())
}
