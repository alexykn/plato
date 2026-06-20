use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PythonPackageManagerConfig {
    Pip,
    Uv,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PythonProjectScopeConfig {
    Requirements,
    Install,
    #[default]
    Base,
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RustProjectScopeConfig {
    Build,
    Fetch,
    #[default]
    Base,
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RustProjectTypeConfig {
    #[serde(alias = "bin")]
    Binary,
    #[serde(alias = "lib")]
    Library,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TemplateLanguage {
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
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) plato: PlatoConfig,
    #[serde(default)]
    pub(crate) template: TemplateConfig,
    #[serde(default)]
    pub(crate) path: PathConfig,
    #[serde(default)]
    pub(crate) python: PythonConfig,
    #[serde(default)]
    pub(crate) rust: RustConfig,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PlatoConfig {
    #[serde(default)]
    pub(crate) template_language: TemplateLanguage,
    #[serde(default)]
    pub(crate) setup_git: bool,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct TemplateConfig {
    #[serde(default)]
    pub(crate) context: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PathConfig {
    #[serde(default)]
    pub(crate) replace: BTreeMap<String, PathReplacementConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PathReplacementConfig {
    pub(crate) path: PathBuf,
    pub(crate) replace: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PythonConfig {
    #[serde(default = "get_default_python_version")]
    pub(crate) language_version: String,
    #[serde(default)]
    pub(crate) package_manager: PythonPackageManagerConfig,
    #[serde(default)]
    pub(crate) project_scope: PythonProjectScopeConfig,
    #[serde(default, rename = "pip")]
    pub(crate) pip_config: PipConfig,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PipConfig {
    #[serde(default)]
    pub(crate) version_fallback: bool,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct RustConfig {
    #[serde(default = "get_default_rust_toolchain")]
    pub(crate) toolchain: String,
    #[serde(default)]
    pub(crate) project_scope: RustProjectScopeConfig,
    #[serde(default)]
    pub(crate) project_type: RustProjectTypeConfig,
    #[serde(default)]
    pub(crate) cargo_init: bool,
}

/// Loads template config from an explicit TOML path.
///
/// # Errors
/// Returns an error if the file is missing, unreadable, or invalid TOML.
pub(crate) fn parse_config_file(toml_path: &Path) -> Result<Config> {
    if !toml_path.exists() {
        let parent = toml_path.parent().unwrap_or_else(|| Path::new("."));
        bail!("Missing plato.toml in {}", parent.display());
    }
    let content = read_to_string(toml_path).context(format!(
        "Could not read plato toml at {}",
        toml_path.display()
    ))?;
    toml::from_str(&content).context(format!(
        "Invalid format in plato toml at {}",
        toml_path.display()
    ))
}
