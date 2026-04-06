use anyhow::Result;
use std::path::PathBuf;

pub mod core;
pub mod languages;
pub mod util;
pub mod workspace;

use crate::core::config::{Config, TemplateLanguage};
use crate::core::guard::ProjectGuard;
use crate::languages::{setup_python_workspace, setup_rust_workspace};
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
    setup_base_workspace(
        &options.project_name,
        &options.config,
        &options.source_path,
        &options.target_path,
    )?;
    match options.config.plato.template_language {
        TemplateLanguage::Python => {
            setup_python_workspace(&options.project_name, &options.config, &options.target_path)?;
            setup_git(&options.target_path)
        }
        TemplateLanguage::Rust => {
            setup_rust_workspace(&options.project_name, &options.config, &options.target_path)
            // we use cargo init to setup rust, no manual git setup needed.
        }
        TemplateLanguage::Base => {
            println!("No supported 'template_language' specified, setting up base workspace.");
            setup_git(&options.target_path)
        }
    }?;
    guard.release();
    Ok(())
}
