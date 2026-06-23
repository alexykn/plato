use anyhow::{Result, bail};
use std::path::{Component, Path, PathBuf};
use toml::Value as TomlValue;

use crate::config::{Config, SetupStepConfig};
use crate::plugins::id::PluginId;
use crate::setup::step::SetupStep;

#[derive(Debug, Clone, Default)]
pub(crate) struct SetupPlan {
    pub(crate) steps: Vec<SetupStep>,
}

impl SetupPlan {
    pub(crate) fn from_config(config: &Config, target_path: &Path) -> Result<Self> {
        let steps = config
            .setup
            .steps
            .iter()
            .map(|step| build_step(config, target_path, step))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { steps })
    }
}

fn build_step(config: &Config, target_path: &Path, step: &SetupStepConfig) -> Result<SetupStep> {
    let plugin = PluginId::parse(step.plugin.clone())?;
    validate_source_path(&step.source_path)?;
    let workdir = normalize_workdir(target_path, &step.source_path)?;
    let plugin_config =
        merge_plugin_config(config.plugins.get(plugin.as_str()), &step.config_overrides);
    let config = serde_json::to_value(plugin_config)?;
    Ok(SetupStep {
        plugin,
        source_path: step.source_path.clone(),
        workdir,
        config,
    })
}

fn merge_plugin_config(
    base: Option<&TomlValue>,
    overrides: &std::collections::BTreeMap<String, TomlValue>,
) -> TomlValue {
    let mut merged = base
        .cloned()
        .unwrap_or_else(|| TomlValue::Table(Default::default()));
    for (key, value) in overrides {
        set_table_value(&mut merged, key.clone(), value.clone());
    }
    merged
}

fn set_table_value(target: &mut TomlValue, key: String, value: TomlValue) {
    match target {
        TomlValue::Table(table) => match table.get_mut(&key) {
            Some(existing) => merge_toml_values(existing, value),
            None => {
                table.insert(key, value);
            }
        },
        _ => {
            let mut table = toml::map::Map::new();
            table.insert(key, value);
            *target = TomlValue::Table(table);
        }
    }
}

fn merge_toml_values(target: &mut TomlValue, source: TomlValue) {
    match (target, source) {
        (TomlValue::Table(target), TomlValue::Table(source)) => {
            for (key, value) in source {
                match target.get_mut(&key) {
                    Some(existing) => merge_toml_values(existing, value),
                    None => {
                        target.insert(key, value);
                    }
                }
            }
        }
        (target, source) => *target = source,
    }
}

fn validate_source_path(path: &Path) -> Result<()> {
    if path.is_absolute() {
        bail!("Setup step source_path {:?} must be relative", path);
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        bail!("Setup step source_path {:?} must not contain '..'", path);
    }
    Ok(())
}

fn normalize_workdir(target_path: &Path, source_path: &Path) -> Result<PathBuf> {
    let workdir = target_path.join(source_path);
    if !workdir.starts_with(target_path) {
        bail!(
            "Setup step workdir escaped target path: {}",
            workdir.display()
        );
    }
    Ok(workdir)
}
