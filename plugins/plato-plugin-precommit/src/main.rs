use anyhow::Context;
use plato_plugin_api::{PluginCapability, PluginMetadata, PluginSetupRequest, PluginSetupResponse};
use plato_plugin_support::{SetupPlugin, command::run_command, run};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct PrecommitConfig {
    #[serde(default)]
    install_hooks: bool,
}
struct PrecommitPlugin;
impl SetupPlugin for PrecommitPlugin {
    fn metadata(&self) -> PluginMetadata {
        metadata("precommit", "Installs pre-commit hooks")
    }
    fn setup(&self, request: PluginSetupRequest) -> anyhow::Result<PluginSetupResponse> {
        let config: PrecommitConfig =
            serde_json::from_value(request.config).context("Invalid precommit plugin config")?;
        if config.install_hooks {
            run_command("pre-commit", ["install"], &request.workdir)?;
        }
        Ok(PluginSetupResponse::success("pre-commit setup complete"))
    }
}
fn metadata(name: &str, description: &str) -> PluginMetadata {
    PluginMetadata {
        name: name.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        supported_api_versions: vec![1],
        capabilities: vec![PluginCapability::Setup],
        description: Some(description.to_string()),
    }
}
fn main() -> std::process::ExitCode {
    run(PrecommitPlugin)
}
