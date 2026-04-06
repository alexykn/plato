use anyhow::{Context, Result, bail};
use directories::BaseDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TemplateLanguage {
    #[serde(alias = "py")]
    Python,
    #[serde(alias = "rs")]
    Rust,
    #[default]
    #[serde(other)]
    Base,
}

fn get_default_version() -> String {
    String::from("latest")
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub plato: PlatoConfig,
    #[serde(default)]
    pub template: TemplateConfig,
}

#[derive(Deserialize, Debug, Default)]
pub struct PlatoConfig {
    #[serde(default)]
    pub template_language: TemplateLanguage,
    #[serde(default = "get_default_version")]
    pub language_version: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct TemplateConfig {
    #[serde(default)]
    pub context: HashMap<String, String>,
}

#[must_use]
pub fn get_config_dir() -> PathBuf {
    let base_dirs = BaseDirs::new().expect("Could not find home directory");
    let mut config_path = base_dirs.home_dir().to_path_buf();
    config_path.push(".config");
    config_path.push("plato");
    config_path
}

pub fn get_config(source: &Path) -> Result<Config> {
    let toml_path = source.join("plato.toml");
    if !toml_path.exists() {
        bail!("Missing plato.toml in {}", source.display());
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
