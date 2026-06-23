use anyhow::{Result, bail};
use std::path::PathBuf;
use std::process::Command;

use crate::plugins::id::PluginId;
use crate::plugins::paths::managed_plugin_root;

#[derive(Debug, Clone)]
pub enum PluginInstallBackend {
    Cargo,
    CargoPath { path: PathBuf },
    Git { url: String },
    UvTool,
    UvToolPath { path: PathBuf },
    Pipx,
    PipxPath { path: PathBuf },
}

pub fn install_plugin(name: &str, backend: PluginInstallBackend) -> Result<()> {
    match backend {
        PluginInstallBackend::Cargo => install_cargo(name),
        PluginInstallBackend::CargoPath { path } => install_cargo_path(&path),
        PluginInstallBackend::Git { url } => install_git(&url),
        PluginInstallBackend::UvTool => {
            run_status(Command::new("uv").arg("tool").arg("install").arg(name))
        }
        PluginInstallBackend::UvToolPath { path } => {
            run_status(Command::new("uv").arg("tool").arg("install").arg(path))
        }
        PluginInstallBackend::Pipx => run_status(Command::new("pipx").arg("install").arg(name)),
        PluginInstallBackend::PipxPath { path } => {
            run_status(Command::new("pipx").arg("install").arg(path))
        }
    }
}

fn install_cargo(name: &str) -> Result<()> {
    let plugin = PluginId::parse(name.to_string())?;
    let crate_name = plugin.binary_name();
    let root = managed_plugin_root()?;
    std::fs::create_dir_all(&root)?;
    run_status(
        Command::new("cargo")
            .arg("install")
            .arg(crate_name)
            .arg("--root")
            .arg(root),
    )
}

fn install_cargo_path(path: &std::path::Path) -> Result<()> {
    let root = managed_plugin_root()?;
    std::fs::create_dir_all(&root)?;
    run_status(
        Command::new("cargo")
            .arg("install")
            .arg("--path")
            .arg(path)
            .arg("--root")
            .arg(root),
    )
}

fn install_git(url: &str) -> Result<()> {
    let root = managed_plugin_root()?;
    std::fs::create_dir_all(&root)?;
    run_status(
        Command::new("cargo")
            .arg("install")
            .arg("--git")
            .arg(url)
            .arg("--root")
            .arg(root),
    )
}

fn run_status(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        bail!("Plugin install command failed");
    }
    Ok(())
}
