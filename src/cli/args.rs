use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "plato")]
#[command(about = "Scaffolds projects from ~/.config/plato", long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Initialize new project from a configured template, --git spec, or --path
    Init(InitArgs),
    /// Validate a template in memory without creating a project or running setup
    Val(ValArgs),
    /// Open a configured template override or local plato.toml in editor
    Config {
        /// The name of the template (e.g., py3.12)
        template_name: String,
    },
    /// List configured templates
    List {
        /// Show source details for every configured template
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Args, Debug)]
pub(crate) struct InitArgs {
    /// Configured template name, or Git spec when --git is passed
    pub(crate) template_name: Option<String>,

    /// The name of the new project directory
    pub(crate) project_name: Option<String>,

    /// Overwrite existing target directory if it exists
    #[arg(short, long)]
    pub(crate) force: bool,

    /// Provide an explicit path to load the template from
    #[arg(short, long)]
    pub(crate) path: Option<PathBuf>,

    /// Treat the template argument as an ad-hoc Git remote spec
    #[arg(long)]
    pub(crate) git: bool,

    /// Git branch, tag, or commit to use for remote templates
    #[arg(long)]
    pub(crate) rev: Option<String>,

    /// Subpath inside a remote repository to use as the template root
    #[arg(long)]
    pub(crate) subpath: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub(crate) struct ValArgs {
    /// Configured template name, or Git spec when --git is passed
    pub(crate) template_name: Option<String>,

    /// The name used for rendering validation context
    pub(crate) project_name: Option<String>,

    /// Provide an explicit path to load the template from
    #[arg(short, long)]
    pub(crate) path: Option<PathBuf>,

    /// Treat the template argument as an ad-hoc Git remote spec
    #[arg(long)]
    pub(crate) git: bool,

    /// Git branch, tag, or commit to use for remote templates
    #[arg(long)]
    pub(crate) rev: Option<String>,

    /// Subpath inside a remote repository to use as the template root
    #[arg(long)]
    pub(crate) subpath: Option<PathBuf>,
}
