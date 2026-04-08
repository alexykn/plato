use anyhow::{Result, bail};
use std::env::current_dir;
use std::{fs::create_dir_all, path::PathBuf};

pub mod core;
pub mod languages;
pub mod util;
pub mod workspace;

use crate::core::config::{Config, TemplateLanguage, get_config, get_global_plato_dir};
use crate::core::guard::ProjectGuard;
use crate::core::registry::TemplateRegistry;
use crate::languages::{LanguageSetup, PythonSetup, RustSetup, SetupContext};
use crate::util::{open_config_file, setup_git};
use crate::workspace::setup_base_workspace;

pub struct RunOptions {
    pub template_name: Option<String>,
    pub project_name: String,
    pub force: bool,
    pub path: Option<PathBuf>,
}

struct ExecutionContext {
    pub project_name: String,
    pub force: bool,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub source_config: Config,
}

impl TryFrom<RunOptions> for ExecutionContext {
    type Error = anyhow::Error;

    fn try_from(options: RunOptions) -> Result<Self, Self::Error> {
        let source_path = if let Some(path) = options.path {
            path
        } else {
            let Some(template_name) = options.template_name.as_deref() else {
                bail!("A template name is required when --path is not provided")
            };
            let global_plato_dir = get_global_plato_dir()?;
            let global_config = get_config(&global_plato_dir).ok();
            let fallback_dirs = Vec::new();
            let extra_template_dirs = if let Some(config) = &global_config {
                &config.plato.extra_dirs
            } else {
                &fallback_dirs
            };
            let registry = TemplateRegistry::build(&global_plato_dir, extra_template_dirs)?;
            let (source_path, _) = registry.get(template_name)?;
            source_path.clone()
        };
        let source_config = get_config(&source_path)?;
        let target_path = current_dir()?.join(&options.project_name);
        Ok(Self {
            project_name: options.project_name,
            force: options.force,
            source_path,
            target_path,
            source_config,
        })
    }
}

/// Opens the selected template config in the user's editor.
///
/// # Errors
/// Returns an error if the global config cannot be loaded, the template cannot be found,
/// or the editor cannot be started successfully.
pub fn edit_config(template_name: &str) -> Result<()> {
    let global_plato_dir = get_global_plato_dir()?;
    let global_config = get_config(&global_plato_dir).ok();
    let fallback_dirs = Vec::new();
    let extra_template_dirs = if let Some(config) = &global_config {
        &config.plato.extra_dirs
    } else {
        &fallback_dirs
    };
    let registry = TemplateRegistry::build(&global_plato_dir, extra_template_dirs)?;
    let selected_config = registry.get_config_path(template_name)?;
    open_config_file(selected_config)
}

/// Displays all discovered templates.
///
/// # Errors
/// Returns an error if the global config cannot be loaded or the template registry cannot be built.
pub fn display_templates() -> Result<()> {
    let global_plato_dir = get_global_plato_dir()?;
    let global_config = get_config(&global_plato_dir).ok();
    let fallback_dirs = Vec::new();
    let extra_template_dirs = if let Some(config) = &global_config {
        &config.plato.extra_dirs
    } else {
        &fallback_dirs
    };
    let registry = TemplateRegistry::build(&global_plato_dir, extra_template_dirs)?;
    registry.display()
}

/// Run the CLI.
///
/// # Errors
/// Returns an error if argument parsing, template loading, filesystem access,
/// template rendering, or project setup fails.
pub fn run(options: RunOptions) -> Result<()> {
    let ctx = ExecutionContext::try_from(options)?;
    if !ctx.force && ctx.target_path.exists() {
        bail!(
            "Target path {} already exists. quitting.",
            ctx.target_path.display()
        )
    }
    let mut guard = ProjectGuard::new(ctx.target_path.clone());
    let should_setup_git: bool = ctx.source_config.plato.setup_git;
    let setup_ctx = SetupContext::from(&ctx);
    create_dir_all(&ctx.target_path)?;
    setup_base_workspace(
        &ctx.project_name,
        &ctx.source_config,
        &ctx.source_path,
        &ctx.target_path,
    )?;

    match &ctx.source_config.plato.template_language {
        TemplateLanguage::Python => PythonSetup.setup(setup_ctx),
        TemplateLanguage::Rust => RustSetup.setup(setup_ctx),
        TemplateLanguage::Base => Ok(()),
    }?;
    if should_setup_git {
        setup_git(&ctx.target_path)?;
    }
    guard.release();
    Ok(())
}
