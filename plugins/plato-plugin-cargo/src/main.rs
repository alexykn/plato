mod config;
mod setup;

use anyhow::Context;
use plato_plugin_api::{PluginCapability, PluginMetadata, PluginSetupRequest, PluginSetupResponse};
use plato_plugin_support::{SetupPlugin, run};

struct CargoPlugin;

impl SetupPlugin for CargoPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "cargo".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            supported_api_versions: vec![1],
            capabilities: vec![PluginCapability::Setup],
            description: Some("Sets up Rust cargo projects".to_string()),
        }
    }

    fn setup(&self, request: PluginSetupRequest) -> anyhow::Result<PluginSetupResponse> {
        let config: config::CargoConfig =
            serde_json::from_value(request.config).context("Invalid cargo plugin config")?;
        setup::setup(&request.workdir, &config)?;
        Ok(PluginSetupResponse::success("cargo setup complete"))
    }
}

fn main() -> std::process::ExitCode {
    run(CargoPlugin)
}
