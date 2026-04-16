use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use plato::{InitSource, RunOptions};

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
        template_name: Option<String>,

        /// The name of the new project directory
        project_name: Option<String>,

        /// Overwrite existing target directory if it exists
        #[arg(short, long)]
        force: bool,

        /// Provide an explicit path to load the template from
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Open the plato.toml for a template in editor
    Config {
        /// The name of the template (e.g., py3.12)
        template_name: String,
    },
    /// List all templates in the template folder
    List,
}

fn map_args(
    template_name: Option<String>,
    project_name: Option<String>,
    path: Option<PathBuf>,
) -> Result<(InitSource, String)> {
    if let Some(template_path) = path {
        let project_name = template_name.ok_or_else(|| {
            anyhow!("When passing --path 'path' only a single additional arg 'project_name' is expected.")
        })?;
        Ok((InitSource::TemplatePath { template_path }, project_name))
    } else {
        let project_name = project_name.ok_or_else(|| {
            anyhow!("Wehn running without --path please pass 'template_name' and 'project_name' as args.")
        })?;
        let template_name = template_name.ok_or_else(|| {
            anyhow!("Wehn running without --path please pass 'template_name' and 'project_name' as args.")
        })?;
        Ok((InitSource::NamedTemplate { template_name }, project_name))
    }
}

fn main() {
    if let Err(error) = try_run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn try_run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            template_name,
            project_name,
            force,
            path,
        } => {
            let (init_source, project_name) = map_args(template_name, project_name, path)?;

            plato::run(RunOptions {
                source: init_source,
                project_name,
                force,
            })
        }
        Commands::Config { template_name } => plato::edit_config(&template_name),
        Commands::List => plato::display_templates(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_args_no_path() {
        let result = map_args(
            Some("template_name".to_string()),
            Some("project_name".to_string()),
            None,
        )
        .unwrap();

        dbg!(&result);

        let InitSource::NamedTemplate { template_name } = result.0 else {
            panic!("Expected InitSource::NamedTemplate");
        };

        assert_eq!(template_name, "template_name");
        assert_eq!(result.1, "project_name");
    }

    #[test]
    fn test_map_args_with_path() {
        let result = map_args(
            Some("template_name".to_string()),
            None,
            Some(PathBuf::from("/some/path")),
        )
        .unwrap();

        dbg!(&result);

        let InitSource::TemplatePath { template_path } = result.0 else {
            panic!("Expected InitSource::TemplatePath");
        };

        assert_eq!(template_path, PathBuf::from("/some/path"));
        assert_eq!(result.1, "template_name");
    }
}
