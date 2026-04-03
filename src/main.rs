use clap::{Parser, Subcommand};
use directories::BaseDirs;
use fs_extra::dir::{CopyOptions, copy};
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

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
        template: String,

        /// The name of the new project directory
        project_name: String,
    },
}

#[derive(Deserialize, Debug)]
struct PlatoToml {
    ttype: String,
    version: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum TemplateType {
    Python,
    Rust,
}

fn get_config_dir() -> PathBuf {
    let base_dirs = BaseDirs::new().expect("Could not find home directory");
    let mut config_path = base_dirs.home_dir().to_path_buf();
    config_path.push(".config");
    config_path.push("plato");
    config_path
}

fn copy_template(source: &PathBuf, target: &PathBuf) -> AppResult<()> {
    let mut options = CopyOptions::new();
    options.content_only = true;

    std::fs::create_dir_all(&target)?;
    copy(&source, &target, &options)?;

    let config_file = target.join("plato.toml");
    if config_file.exists() {
        std::fs::remove_file(config_file)?;
    }
    Ok(())
}

fn is_installed(cmd: &str) -> bool {
    match std::process::Command::new(cmd).arg("--help").output() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn setup_python(version: &str, target: &PathBuf) -> AppResult<()> {
    if is_installed("uv") {
        std::process::Command::new("uv")
            .args(["venv", "--python", version])
            .current_dir(&target)
            .status()?;
        Ok(())
    } else if is_installed("python") {
        let cmd = format!("python{}", version);
        std::process::Command::new(cmd)
            .args(["-m", "venv", ".venv"])
            .current_dir(&target)
            .status()?;
        Ok(())
    } else {
        return Err(format!(
            "Neither 'uv' nor 'python{}' was found on your system. Aborting.",
            version
        )
        .into());
    }
}

fn setup_workspace(ttype: TemplateType, version: &String, target: &PathBuf) -> AppResult<()> {
    match ttype {
        TemplateType::Python => setup_python(version, target)?,
        TemplateType::Rust => println!("Not Implemented!"),
    }
    Ok(())
}

fn setup_git(target: &PathBuf) -> AppResult<()> {
    std::process::Command::new("git")
        .arg("init")
        .current_dir(&target)
        .status()?;
    Ok(())
}

fn parse_template(source: &PathBuf) -> AppResult<(TemplateType, String)> {
    let toml_path = source.join("plato.toml");
    if !toml_path.exists() {
        return Err(format!("Missing plato.toml in {:?}", source).into());
    }

    let content = std::fs::read_to_string(toml_path)?;
    let config: PlatoToml = toml::from_str(&content)?;

    let ttype = match config.ttype.to_lowercase().as_str() {
        "python" | "py" => TemplateType::Python,
        "rust" | "rs" => TemplateType::Rust,
        _ => return Err(format!("Unknown template_type in toml: {}", config.ttype).into()),
    };

    let version = config.version.unwrap_or_else(|| String::from("latest"));

    Ok((ttype, version))
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();
    let pwd = env::current_dir()?;

    match &cli.command {
        Commands::Init {
            template,
            project_name,
        } => {
            let target = pwd.join(project_name);
            let source = get_config_dir().join(template);
            let (ttype, version) = parse_template(&source)?;
            copy_template(&source, &target)?;
            setup_workspace(ttype, &version, &target)?;
            setup_git(&target)?;
            Ok(())
        }
    }
}
