use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PythonPackageManagerConfig {
    Pip,
    #[default]
    Uv,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PythonProjectScopeConfig {
    Requirements,
    Install,
    #[default]
    Base,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PythonUvSetupConfig {
    #[default]
    Editable,
    Sync,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RustProjectScopeConfig {
    Build,
    Fetch,
    #[default]
    Base,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RustProjectTypeConfig {
    #[default]
    #[serde(alias = "bin")]
    Binary,
    #[serde(alias = "lib")]
    Library,
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
    pub(crate) git: GitConfig,
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
    pub(crate) context: BTreeMap<String, Value>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PathConfig {
    #[serde(default)]
    pub(crate) replace: BTreeMap<String, PathReplacementConfig>,
    #[serde(default)]
    pub(crate) exclude: BTreeMap<String, PathExcludeConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PathReplacementConfig {
    pub(crate) path: PathBuf,
    pub(crate) replace: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PathExcludeConfig {
    pub(crate) path: PathBuf,
    #[serde(default)]
    pub(crate) unless: Option<String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct GroupConfig {
    #[serde(default)]
    pub(crate) template: TemplateConfig,
    #[serde(default)]
    pub(crate) path: PathConfig,
}

fn get_default_initial_commit_message() -> String {
    String::from("Initial commit")
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct GitConfig {
    #[serde(default)]
    pub(crate) initial_branch: Option<String>,
    #[serde(default)]
    pub(crate) user: GitUserConfig,
    #[serde(default)]
    pub(crate) commit: GitCommitConfig,
    #[serde(default)]
    pub(crate) core: GitCoreConfig,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct GitUserConfig {
    #[serde(default)]
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) email: Option<String>,
    #[serde(default)]
    pub(crate) signing_key: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct GitCommitConfig {
    #[serde(default)]
    pub(crate) gpgsign: Option<bool>,
    #[serde(default)]
    pub(crate) initial: bool,
    #[serde(default = "get_default_initial_commit_message")]
    pub(crate) initial_message: String,
}

impl Default for GitCommitConfig {
    fn default() -> Self {
        Self {
            gpgsign: None,
            initial: false,
            initial_message: get_default_initial_commit_message(),
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct GitCoreConfig {
    #[serde(default)]
    pub(crate) hooks_path: Option<PathBuf>,
    #[serde(default)]
    pub(crate) autocrlf: Option<GitAutoCrlfConfig>,
    #[serde(default)]
    pub(crate) eol: Option<GitEolConfig>,
    #[serde(default)]
    pub(crate) filemode: Option<bool>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub(crate) enum GitAutoCrlfConfig {
    Bool(bool),
    Mode(GitAutoCrlfMode),
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum GitAutoCrlfMode {
    Input,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum GitEolConfig {
    Lf,
    Crlf,
    Native,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PythonConfig {
    #[serde(default = "get_default_python_version")]
    pub(crate) language_version: String,
    #[serde(default)]
    pub(crate) package_manager: PythonPackageManagerConfig,
    #[serde(default)]
    pub(crate) project_scope: PythonProjectScopeConfig,
    #[serde(default)]
    pub(crate) install: PythonInstallConfig,
    #[serde(default, rename = "uv")]
    pub(crate) uv_config: Option<UvConfig>,
}

impl PythonConfig {
    pub(crate) fn uv_setup(&self) -> PythonUvSetupConfig {
        self.uv_config
            .map(|config| config.setup)
            .unwrap_or_default()
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PythonInstallConfig {
    #[serde(default)]
    pub(crate) groups: Vec<String>,
    #[serde(default)]
    pub(crate) extras: Vec<String>,
}

#[derive(Deserialize, Debug, Default, Clone, Copy)]
pub(crate) struct UvConfig {
    #[serde(default)]
    pub(crate) setup: PythonUvSetupConfig,
}

impl PythonInstallConfig {
    pub(crate) fn is_empty(&self) -> bool {
        self.groups.is_empty() && self.extras.is_empty()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RustConfig {
    #[serde(default = "get_default_rust_toolchain")]
    pub(crate) toolchain: String,
    #[serde(default)]
    pub(crate) components: Vec<String>,
    #[serde(default)]
    pub(crate) targets: Vec<String>,
    #[serde(default)]
    pub(crate) project_scope: RustProjectScopeConfig,
    #[serde(default)]
    pub(crate) project_type: RustProjectTypeConfig,
    #[serde(default)]
    pub(crate) cargo_init: bool,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            toolchain: get_default_rust_toolchain(),
            components: Vec::new(),
            targets: Vec::new(),
            project_scope: RustProjectScopeConfig::default(),
            project_type: RustProjectTypeConfig::default(),
            cargo_init: false,
        }
    }
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
