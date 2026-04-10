use anyhow::{Context, Result, bail};
use directories::BaseDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PythonPackageManagerConfig {
    Pip,
    Uv,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PythonProjectScopeConfig {
    Requirements,
    Install,
    #[default]
    Base,
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RustProjectScopeConfig {
    Build,
    Fetch,
    #[default]
    Base,
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RustProjectTypeConfig {
    #[serde(alias = "bin")]
    Binary,
    #[serde(alias = "lib")]
    Library,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TemplateLanguage {
    #[serde(alias = "py")]
    Python,
    #[serde(alias = "rs")]
    Rust,
    #[default]
    Base,
}

fn get_default_python_version() -> String {
    String::from("3")
}

fn get_default_rust_toolchain() -> String {
    String::from("stable")
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub plato: PlatoConfig,
    #[serde(default)]
    pub template: TemplateConfig,
    #[serde(default)]
    pub python: PythonConfig,
    #[serde(default)]
    pub rust: RustConfig,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PlatoConfig {
    #[serde(default)]
    pub template_language: TemplateLanguage,
    #[serde(default)]
    pub setup_git: bool,
    #[serde(default)]
    pub extra_dirs: Vec<PathBuf>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct TemplateConfig {
    #[serde(default)]
    pub context: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PythonConfig {
    #[serde(default = "get_default_python_version")]
    pub language_version: String,
    #[serde(default)]
    pub package_manager: PythonPackageManagerConfig,
    #[serde(default)]
    pub project_scope: PythonProjectScopeConfig,
    #[serde(default, rename = "pip")]
    pub pip_config: PipConfig,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PipConfig {
    #[serde(default)]
    pub version_fallback: bool,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct RustConfig {
    #[serde(default = "get_default_rust_toolchain")]
    pub toolchain: String,
    #[serde(default)]
    pub project_scope: RustProjectScopeConfig,
    #[serde(default)]
    pub project_type: RustProjectTypeConfig,
    #[serde(default)]
    pub cargo_init: bool,
}

/// Returns the directory that stores Plato's configuration files.
///
/// # Errors
/// Returns an error if the user's home directory cannot be determined.
pub(crate) fn get_global_plato_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Could not find home directory")?;
    let mut config_path = base_dirs.home_dir().to_path_buf();
    config_path.push(".config");
    config_path.push("plato");
    Ok(config_path)
}

/// Loads `plato.toml` from the provided source directory.
///
/// # Errors
/// Returns an error if the file is missing, unreadable, or invalid TOML.
pub(crate) fn get_config(source_path: &Path) -> Result<Config> {
    let toml_path = source_path.join("plato.toml");
    if !toml_path.exists() {
        bail!("Missing plato.toml in {}", source_path.display());
    }
    let content = read_to_string(&toml_path).context(format!(
        "Could not read plato toml at {}",
        toml_path.display()
    ))?;
    let config: Config = toml::from_str(&content).context(format!(
        "Invalid format in plato toml at {}",
        toml_path.display()
    ))?;

    Ok(config)
}
