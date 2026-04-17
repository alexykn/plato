use anyhow::Result;
use std::env::current_dir;
use std::path::PathBuf;

pub(crate) mod core;
pub(crate) mod languages;
pub(crate) mod util;
pub(crate) mod workspace;

use crate::core::config::{Config, TemplateLanguage, get_global_plato_dir, parse_config};
use crate::core::guard::ProjectGuard;
use crate::core::registry::TemplateRegistry;
use crate::languages::{LanguageSetup, LanguageSetupContext, PythonSetup, RustSetup};
use crate::util::{
    bail_if_target_path_exists, get_source_path_for_template, open_config_file, setup_git,
};
use crate::workspace::DefaultWorkspaceSetup;
use crate::workspace::{WorkspaceSetup, WorkspaceSetupContext};

#[derive(Clone, Debug)]
pub enum InitSource {
    NamedTemplate { template_name: String },
    TemplatePath { template_path: PathBuf },
}

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub project_name: String,
    pub source: InitSource,
    pub force: bool,
}

#[derive(Debug, Clone)]
struct ExecutionContext {
    project_name: String,
    force: bool,
    source_path: PathBuf,
    target_path: PathBuf,
    config: Config,
}

impl TryFrom<RunOptions> for ExecutionContext {
    type Error = anyhow::Error;

    fn try_from(options: RunOptions) -> Result<Self, Self::Error> {
        let source_path = match options.source {
            InitSource::TemplatePath { template_path } => template_path,
            InitSource::NamedTemplate { template_name } => {
                get_source_path_for_template(template_name.as_str())?
            }
        };
        let source_config = parse_config(&source_path)?;
        let target_path = current_dir()?.join(&options.project_name);
        Ok(Self {
            project_name: options.project_name,
            force: options.force,
            source_path,
            target_path,
            config: source_config,
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
    let global_config = parse_config(&global_plato_dir).ok();
    let fallback_dirs = Vec::new();
    let extra_template_dirs = if let Some(config) = &global_config {
        &config.plato.extra_dirs
    } else {
        &fallback_dirs
    };
    let registry = TemplateRegistry::build(&global_plato_dir, extra_template_dirs);
    let selected_config = registry.get_config_path(template_name)?;
    open_config_file(selected_config)
}

/// Displays all discovered templates.
///
/// # Errors
/// Returns an error if the global config cannot be loaded or the template registry cannot be built.
pub fn display_templates() -> Result<()> {
    let global_plato_dir = get_global_plato_dir()?;
    let global_config = parse_config(&global_plato_dir).ok();
    let fallback_dirs = Vec::new();
    let extra_template_dirs = if let Some(config) = &global_config {
        &config.plato.extra_dirs
    } else {
        &fallback_dirs
    };
    let registry = TemplateRegistry::build(&global_plato_dir, extra_template_dirs);
    print!("{registry}");
    Ok(())
}

/// Run the CLI.
///
/// # Errors
/// Returns an error if argument parsing, template loading, filesystem access,
/// template rendering, or project setup fails.
pub fn run(options: RunOptions) -> Result<()> {
    let exec_ctx = ExecutionContext::try_from(options)?;
    let target_path = exec_ctx.target_path.clone();
    let should_setup_git = exec_ctx.config.plato.setup_git;
    bail_if_target_path_exists(&exec_ctx.target_path, exec_ctx.force)?;

    let mut guard = ProjectGuard::new(exec_ctx.target_path.clone());
    run_workspace_setup(exec_ctx.clone(), &DefaultWorkspaceSetup)?;
    match &exec_ctx.config.plato.template_language {
        TemplateLanguage::Python => run_language_setup(exec_ctx, &PythonSetup),
        TemplateLanguage::Rust => run_language_setup(exec_ctx, &RustSetup),
        TemplateLanguage::Base => Ok(()),
    }?;
    if should_setup_git {
        setup_git(&target_path)?;
    }
    guard.release();
    Ok(())
}

fn run_language_setup<L>(exec_ctx: ExecutionContext, language_setup: &L) -> Result<()>
where
    L: LanguageSetup,
{
    let language_ctx = LanguageSetupContext::from(exec_ctx);
    language_setup.setup(language_ctx)?;
    Ok(())
}

fn run_workspace_setup<W>(exec_ctx: ExecutionContext, workspace_setup: &W) -> Result<()>
where
    W: WorkspaceSetup,
{
    let workspace_ctx = WorkspaceSetupContext::from(exec_ctx);
    workspace_setup.setup(workspace_ctx)?;
    Ok(())
}
