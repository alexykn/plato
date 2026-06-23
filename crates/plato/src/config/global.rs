use anyhow::{Context, Result};
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
    #[serde(default)]
    pub(crate) plugin_registry: HashMap<String, PluginRegistryEntry>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PluginRegistryEntry {
    pub(crate) command: PathBuf,
    #[serde(default)]
    pub(crate) source: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct GlobalPlatoConfig {
    #[serde(default)]
    pub(crate) default_git_provider: GitProvider,
    #[serde(default)]
    pub(crate) git_hosts: GitHostsConfig,
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
