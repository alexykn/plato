pub(crate) mod global;
pub(crate) mod template;
#[cfg(test)]
mod tests;

pub(crate) use global::{
    GitProvider, GlobalConfig, TemplateEntry, get_global_config_path, parse_global_config_file,
};
pub(crate) use template::{
    Config, GitAutoCrlfConfig, GitAutoCrlfMode, GitEolConfig, PathReplacementConfig,
    PythonInstallConfig, PythonPackageManagerConfig, PythonProjectScopeConfig, PythonUvSetupConfig,
    RustProjectScopeConfig, RustProjectTypeConfig, TemplateLanguage, parse_config_file,
};
