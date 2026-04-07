use clap::{Parser, Subcommand};
use std::env;

use plato::RunOptions;
use plato::core::config::{get_config, get_config_dir};
use plato::util::{list_templates, open_config_file};

/// Plato: A cool project templating tool
#[derive(Parser, Debug)]
#[command(name = "plato")]
#[command(about = "Scaffolds projects from ~/.config/plato", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize new project from a template
    Init {
        /// The name of the template (e.g., py3.12)
        template_name: String,

        /// The name of the new project directory
        project_name: String,
    },
    /// Open the plato.toml for a template in editor
    Config {
        /// The name of the template (e.g., py3.12)
        template_name: String,
    },
    /// List all templates in the template folder
    List,
}

fn main() {
    if let Err(error) = try_run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn try_run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let pwd = env::current_dir()?;

    match cli.command {
        Commands::Init {
            template_name,
            project_name,
        } => {
            let source_path = get_config_dir()?.join(&template_name);
            let config = get_config(&source_path)?;
            let target_path = pwd.join(&project_name);
            plato::run(&RunOptions {
                template_name,
                project_name,
                source_path,
                target_path,
                config,
            })
        }
        Commands::Config { template_name } => {
            let source_path = get_config_dir()?.join(&template_name);
            open_config_file(&source_path)
        }
        Commands::List => {
            let config_dir = get_config_dir()?;
            list_templates(&config_dir)
        }
    }
}
