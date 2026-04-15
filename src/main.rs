use anyhow::{Result, anyhow, bail};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use plato::RunOptions;

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
    path: Option<&PathBuf>,
) -> Result<(Option<String>, String)> {
    if path.is_some() {
        if project_name.is_some() {
            bail!("With --path, init expects exactly one positional argument: <PROJECT_NAME>");
        }
        let project_name = template_name.ok_or_else(|| {
            anyhow!("With --path, init expects exactly one positional argument: <PROJECT_NAME>")
        })?;
        return Ok((None, project_name));
    }

    let template_name = template_name
        .ok_or_else(|| anyhow!("Without --path, init expects <TEMPLATE_NAME> <PROJECT_NAME>"))?;
    let project_name = project_name
        .ok_or_else(|| anyhow!("Without --path, init expects <TEMPLATE_NAME> <PROJECT_NAME>"))?;
    Ok((Some(template_name), project_name))
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
            let (template_name, project_name) =
                map_args(template_name, project_name, path.as_ref())?;

            plato::run(RunOptions {
                template_name,
                project_name,
                force,
                path,
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

        assert_eq!(result.0.unwrap(), "template_name");
        assert_eq!(result.1, "project_name");
    }

    #[test]
    fn test_map_args_with_path() {
        let result = map_args(
            Some("template_name".to_string()),
            None,
            Some(&PathBuf::from("/some/path")),
        )
        .unwrap();

        dbg!(&result);

        assert!(result.0.is_none());
        assert_eq!(result.1, "template_name");
    }
}
