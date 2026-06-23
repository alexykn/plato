use anyhow::{Result, bail};
use plato_plugin_support::command::run_command;
use std::path::Path;

use crate::config::{CargoConfig, CargoScope, RustProjectType};

pub(crate) fn setup(workdir: &Path, config: &CargoConfig) -> Result<()> {
    validate(workdir, config)?;
    ensure_toolchain(workdir, config)?;
    add_components(workdir, config)?;
    add_targets(workdir, config)?;
    if config.cargo_init {
        cargo_init(workdir, config)?;
    }
    match config.scope {
        CargoScope::Base => Ok(()),
        CargoScope::Fetch => cargo_command(workdir, config, "fetch"),
        CargoScope::Build => cargo_command(workdir, config, "build"),
    }
}

fn validate(workdir: &Path, config: &CargoConfig) -> Result<()> {
    if config.cargo_init && workdir.join("Cargo.toml").exists() {
        bail!(
            "cannot run cargo init because Cargo.toml already exists; remove cargo_init or remove the manifest"
        );
    }
    Ok(())
}

fn ensure_toolchain(workdir: &Path, config: &CargoConfig) -> Result<()> {
    run_command(
        "rustup",
        ["toolchain", "install", config.toolchain.as_str()],
        workdir,
    )
}

fn add_components(workdir: &Path, config: &CargoConfig) -> Result<()> {
    if config.components.is_empty() {
        return Ok(());
    }
    let mut args = vec!["component".to_string(), "add".to_string()];
    args.extend(config.components.iter().cloned());
    args.extend(["--toolchain".to_string(), config.toolchain.clone()]);
    run_command("rustup", args, workdir)
}

fn add_targets(workdir: &Path, config: &CargoConfig) -> Result<()> {
    if config.targets.is_empty() {
        return Ok(());
    }
    let mut args = vec!["target".to_string(), "add".to_string()];
    args.extend(config.targets.iter().cloned());
    args.extend(["--toolchain".to_string(), config.toolchain.clone()]);
    run_command("rustup", args, workdir)
}

fn cargo_init(workdir: &Path, config: &CargoConfig) -> Result<()> {
    let project_type = match config.project_type {
        RustProjectType::Binary => "--bin",
        RustProjectType::Library => "--lib",
    };
    run_command(
        "cargo",
        [
            format!("+{}", config.toolchain),
            "init".to_string(),
            project_type.to_string(),
            "--vcs".to_string(),
            "none".to_string(),
        ],
        workdir,
    )
}

fn cargo_command(workdir: &Path, config: &CargoConfig, command: &str) -> Result<()> {
    run_command(
        "cargo",
        [format!("+{}", config.toolchain), command.to_string()],
        workdir,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all, write};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rejects_cargo_init_when_manifest_exists() {
        let dir = std::env::temp_dir().join(format!(
            "plato-cargo-plugin-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        create_dir_all(&dir).unwrap();
        write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        let config = CargoConfig {
            cargo_init: true,
            ..CargoConfig::default()
        };
        assert!(validate(&dir, &config).is_err());
        remove_dir_all(dir).unwrap();
    }
}
