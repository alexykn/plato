use std::path::PathBuf;

use crate::plugins::id::PluginId;

#[derive(Debug, Clone)]
pub(crate) struct SetupStep {
    pub(crate) plugin: PluginId,
    pub(crate) source_path: PathBuf,
    pub(crate) workdir: PathBuf,
    pub(crate) config: serde_json::Value,
}
