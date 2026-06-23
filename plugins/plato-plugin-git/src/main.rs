mod config;
mod setup;

use anyhow::Context;
use plato_plugin_api::{PluginCapability, PluginMetadata, PluginSetupRequest, PluginSetupResponse};
use plato_plugin_support::{SetupPlugin, run};

struct GitPlugin;

impl SetupPlugin for GitPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "git".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            supported_api_versions: vec![1],
            capabilities: vec![PluginCapability::Setup],
            description: Some("Initializes git repositories for Plato projects".to_string()),
        }
    }

    fn setup(&self, request: PluginSetupRequest) -> anyhow::Result<PluginSetupResponse> {
        let config: config::GitPluginConfig =
            serde_json::from_value(request.config).context("Invalid git plugin config")?;
        setup::run(&request.workdir, &config)?;
        Ok(PluginSetupResponse::success("git setup complete"))
    }
}

fn main() -> std::process::ExitCode {
    run(GitPlugin)
}
