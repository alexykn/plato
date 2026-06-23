use anyhow::{Context, Result, bail};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};
use toml::Value;

use crate::config::get_global_config_path;
use crate::plugins::id::PluginId;

pub fn register_plugin(name: &str, command: &Path) -> Result<()> {
    let plugin = PluginId::parse(name.to_string())?;
    let path = get_global_config_path()?;
    let mut root = load_global_toml(&path)?;
    let table = root
        .as_table_mut()
        .context("Global config root must be a TOML table")?;
    let registry = table
        .entry("plugin_registry".to_string())
        .or_insert_with(|| Value::Table(Default::default()))
        .as_table_mut()
        .context("[plugin_registry] must be a TOML table")?;
    let mut entry = toml::map::Map::new();
    entry.insert(
        "command".to_string(),
        Value::String(command.to_string_lossy().to_string()),
    );
    entry.insert("source".to_string(), Value::String("manual".to_string()));
    registry.insert(plugin.as_str().to_string(), Value::Table(entry));
    write_global_toml(&path, root)
}

pub fn remove_plugin(name: &str) -> Result<()> {
    let plugin = PluginId::parse(name.to_string())?;
    let path = get_global_config_path()?;
    let mut root = load_global_toml(&path)?;
    let table = root
        .as_table_mut()
        .context("Global config root must be a TOML table")?;
    let Some(registry) = table
        .get_mut("plugin_registry")
        .and_then(Value::as_table_mut)
    else {
        return Ok(());
    };
    registry.remove(plugin.as_str());
    write_global_toml(&path, root)
}

fn load_global_toml(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Table(Default::default()));
    }
    let raw = read_to_string(path)
        .with_context(|| format!("Could not read global config at {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(Value::Table(Default::default()));
    }
    toml::from_str(&raw).with_context(|| format!("Invalid global config at {}", path.display()))
}

fn write_global_toml(path: &Path, root: Value) -> Result<()> {
    let Some(parent) = path.parent() else {
        bail!("Global config path {} has no parent", path.display());
    };
    create_dir_all(parent)?;
    write(path, toml::to_string_pretty(&root)?)?;
    Ok(())
}

#[allow(dead_code)]
fn _debug_path(path: PathBuf) -> PathBuf {
    path
}
