use anyhow::Result;
use std::env::current_dir;
use std::path::PathBuf;

pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod fs;
pub(crate) mod git;
pub(crate) mod guard;
pub(crate) mod languages;
pub(crate) mod names;
pub(crate) mod rendering;
pub(crate) mod templates;
pub(crate) mod util;
pub(crate) mod workspace;

use crate::config::Config;
use crate::config::TemplateLanguage;
use crate::config::group::apply_group_configs;
use crate::context::{ContextMap, ContextOverrides};
use crate::git::{
    GitCommitLocalConfig, GitCoreConfig, GitInitialCommit, GitLocalConfig, GitSetupOptions,
    GitUserConfig, TempCheckout, setup_git_repository,
};
use crate::guard::ProjectGuard;
use crate::languages::{LanguageSetup, LanguageSetupContext, PythonSetup, RustSetup};
use crate::templates::{TemplateRequest, TemplateResolver};
use crate::util::{bail_if_target_path_exists, open_config_file};
use crate::workspace::{WorkspaceRenderContext, render_workspace};

#[derive(Clone, Debug)]
pub enum InitSource {
    NamedTemplate { template_name: String },
    GitTemplate { git_spec: String },
    TemplatePath { template_path: PathBuf },
}

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub project_name: String,
    pub source: InitSource,
    pub force: bool,
    pub rev: Option<String>,
    pub subpath: Option<PathBuf>,
    pub groups: Vec<String>,
    pub set_values: Vec<String>,
    pub set_string_values: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ValidateOptions {
    pub project_name: String,
    pub source: InitSource,
    pub rev: Option<String>,
    pub subpath: Option<PathBuf>,
    pub groups: Vec<String>,
    pub set_values: Vec<String>,
    pub set_string_values: Vec<String>,
}

struct PreparedTemplateContext {
    project_name: String,
    source_path: PathBuf,
    config: Config,
    context_overrides: ContextMap,
    source_cleanup: Option<TempCheckout>,
}

struct ExecutionContext {
    project_name: String,
    force: bool,
    source_path: PathBuf,
    target_path: PathBuf,
    config: Config,
    context_overrides: ContextMap,
    _source_cleanup: Option<TempCheckout>,
}

impl TryFrom<RunOptions> for ExecutionContext {
    type Error = anyhow::Error;

    fn try_from(options: RunOptions) -> Result<Self, Self::Error> {
        let prepared = prepare_template_context(
            options.source,
            options.project_name,
            options.rev,
            options.subpath,
            options.groups,
            options.set_values,
            options.set_string_values,
        )?;
        let target_path = current_dir()?.join(&prepared.project_name);
        Ok(Self {
            project_name: prepared.project_name,
            force: options.force,
            source_path: prepared.source_path,
            target_path,
            config: prepared.config,
            context_overrides: prepared.context_overrides,
            _source_cleanup: prepared.source_cleanup,
        })
    }
}

fn prepare_template_context(
    source: InitSource,
    project_name: String,
    rev: Option<String>,
    subpath: Option<PathBuf>,
    groups: Vec<String>,
    set_values: Vec<String>,
    set_string_values: Vec<String>,
) -> Result<PreparedTemplateContext> {
    let resolver = TemplateResolver::from_global_config()?;
    let prepared_source = match source {
        InitSource::TemplatePath { template_path } => resolver.prepare(TemplateRequest::Path {
            path: template_path,
        })?,
        InitSource::GitTemplate { git_spec } => resolver.prepare(TemplateRequest::Git {
            spec: git_spec,
            cli_rev: rev,
            cli_subpath: subpath,
        })?,
        InitSource::NamedTemplate { template_name } => {
            resolver.prepare(TemplateRequest::Named {
                name: template_name,
                cli_rev: rev,
                cli_subpath: subpath,
            })?
        }
    };

    let mut config = prepared_source.config;
    apply_group_configs(&mut config, &prepared_source.source_path, &groups)?;
    let context_overrides = ContextOverrides::parse(&set_values, &set_string_values)?.into_values();

    Ok(PreparedTemplateContext {
        project_name,
        source_path: prepared_source.source_path,
        config,
        context_overrides,
        source_cleanup: prepared_source.cleanup,
    })
}

/// Opens the selected template config in the user's editor.
///
/// # Errors
/// Returns an error if the global config cannot be loaded, the template cannot be found,
/// or the editor cannot be started successfully.
pub fn edit_config(template_name: &str) -> Result<()> {
    let resolver = TemplateResolver::from_global_config()?;
    let config_path = resolver.config_path_for(template_name)?;
    open_config_file(&config_path)
}

/// Displays all configured templates.
///
/// # Errors
/// Returns an error if the global config cannot be loaded or the template registry cannot be built.
pub fn display_templates(verbose: bool) -> Result<()> {
    let resolver = TemplateResolver::from_global_config()?;
    print!("{}", resolver.format_templates(verbose));
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

    let rendered = render_workspace(&WorkspaceRenderContext::from(&exec_ctx))?;

    let mut guard = ProjectGuard::new(exec_ctx.target_path.clone());
    rendered.flush_to_disk(&exec_ctx.target_path)?;
    match &exec_ctx.config.plato.template_language {
        TemplateLanguage::Python => run_language_setup(&exec_ctx, &PythonSetup),
        TemplateLanguage::Rust => run_language_setup(&exec_ctx, &RustSetup),
        TemplateLanguage::Base => Ok(()),
    }?;
    if should_setup_git {
        setup_git_repository(&target_path, git_setup_options(&exec_ctx.config))?;
    }
    guard.release();
    Ok(())
}

/// Validate a template without writing a project or running setup commands.
///
/// # Errors
/// Returns an error if source resolution or rendering fails.
pub fn validate(options: ValidateOptions) -> Result<()> {
    let prepared = prepare_template_context(
        options.source,
        options.project_name,
        options.rev,
        options.subpath,
        options.groups,
        options.set_values,
        options.set_string_values,
    )?;
    let render_ctx = WorkspaceRenderContext::from(&prepared);
    render_workspace(&render_ctx)?;
    println!("Validation passed.");
    Ok(())
}

fn git_setup_options(config: &Config) -> GitSetupOptions<'_> {
    let git = &config.git;

    GitSetupOptions {
        initial_branch: git.initial_branch.as_deref(),
        local_config: GitLocalConfig {
            user: GitUserConfig {
                name: git.user.name.as_deref(),
                email: git.user.email.as_deref(),
                signing_key: git.user.signing_key.as_deref(),
            },
            commit: GitCommitLocalConfig {
                gpgsign: git.commit.gpgsign,
            },
            core: GitCoreConfig {
                hooks_path: git.core.hooks_path.as_deref(),
                autocrlf: git.core.autocrlf.map(Into::into),
                eol: git.core.eol.map(Into::into),
                filemode: git.core.filemode,
            },
        },
        initial_commit: git.commit.initial.then_some(GitInitialCommit {
            message: &git.commit.initial_message,
        }),
    }
}

fn run_language_setup<L>(exec_ctx: &ExecutionContext, language_setup: &L) -> Result<()>
where
    L: LanguageSetup,
{
    let language_ctx = LanguageSetupContext::from(exec_ctx);
    language_setup.setup(language_ctx)?;
    Ok(())
}

impl From<&PreparedTemplateContext> for WorkspaceRenderContext {
    fn from(ctx: &PreparedTemplateContext) -> Self {
        Self {
            template_context: workspace::build_template_context_parts(
                &ctx.project_name,
                &ctx.config,
                ctx.context_overrides.clone(),
            ),
            path_replacements: ctx.config.path.replace.clone(),
            path_excludes: ctx.config.path.exclude.clone(),
            source_path: ctx.source_path.clone(),
        }
    }
}
