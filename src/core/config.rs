use anyhow::{Context, Result, bail};
use directories::BaseDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum GitProvider {
    #[default]
    Github,
    Gitlab,
    Bitbucket,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct GitHostsConfig {
    #[serde(default)]
    pub(crate) github: Option<String>,
    #[serde(default)]
    pub(crate) gitlab: Option<String>,
    #[serde(default)]
    pub(crate) bitbucket: Option<String>,
}

impl GitHostsConfig {
    pub(crate) fn get(&self, provider: GitProvider) -> Option<&str> {
        match provider {
            GitProvider::Github => self.github.as_deref(),
            GitProvider::Gitlab => self.gitlab.as_deref(),
            GitProvider::Bitbucket => self.bitbucket.as_deref(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum TemplateEntry {
    Path {
        path: PathBuf,
    },
    Git {
        git: String,
        #[serde(default)]
        rev: Option<String>,
        #[serde(default)]
        subpath: Option<PathBuf>,
    },
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct GlobalConfig {
    #[serde(default)]
    pub(crate) plato: GlobalPlatoConfig,
    #[serde(default)]
    pub(crate) templates: HashMap<String, TemplateEntry>,
    #[serde(default)]
    pub(crate) template_configs: HashMap<String, PathBuf>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct GlobalPlatoConfig {
    #[serde(default)]
    pub(crate) default_git_provider: GitProvider,
    #[serde(default)]
    pub(crate) git_hosts: GitHostsConfig,
    #[serde(default = "get_default_remote_config_dir")]
    pub(crate) remote_config_dir: PathBuf,
}

impl Default for GlobalPlatoConfig {
    fn default() -> Self {
        Self {
            default_git_provider: GitProvider::default(),
            git_hosts: GitHostsConfig::default(),
            remote_config_dir: get_default_remote_config_dir(),
        }
    }
}

fn get_default_remote_config_dir() -> PathBuf {
    PathBuf::from("~/.config/plato/remote_configs")
}

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

/// Returns the directory that stores Plato's configuration files.
///
/// # Errors
/// Returns an error if the user's home directory cannot be determined.
pub(crate) fn get_global_plato_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Could not find home directory")?;
    Ok(base_dirs.home_dir().join(".config/plato"))
}

/// Returns the global Plato config path.
///
/// # Errors
/// Returns an error if the user's home directory cannot be determined.
pub(crate) fn get_global_config_path() -> Result<PathBuf> {
    Ok(get_global_plato_dir()?.join("config.toml"))
}

/// Loads global config from an explicit TOML path.
///
/// # Errors
/// Returns an error if the file is unreadable or invalid TOML.
pub(crate) fn parse_global_config_file(toml_path: &Path) -> Result<GlobalConfig> {
    let content = read_to_string(toml_path).context(format!(
        "Could not read global config at {}",
        toml_path.display()
    ))?;
    toml::from_str(&content).context(format!(
        "Invalid format in global config at {}",
        toml_path.display()
    ))
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
