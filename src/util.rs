use anyhow::{Context, Result, bail};
use regex::Regex;
use std::env::var;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use crate::{TemplateRegistry, get_config, get_global_plato_dir};

static ALLOWED_CMD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(git|cargo|uv|python\d*(?:\.\d+)*)$").expect("Invalid regex pattern")
});

fn get_default_editor() -> Result<(String, Vec<String>)> {
    let visual = var("VISUAL").unwrap_or_default();
    let editor = var("EDITOR").unwrap_or_default();
    let raw_cmd = if !visual.trim().is_empty() {
        &visual
    } else if !editor.trim().is_empty() {
        &editor
    } else {
        "nano"
    };
    let mut parts = shell_words::split(raw_cmd)
        .context("Could not parse EDITOR/VISUAL command.")?
        .into_iter();

    let command = parts.next().unwrap_or_else(|| "nano".to_string());
    let args = parts.collect();
    Ok((command, args))
}

/// Opens the template's `plato.toml` in the user's editor.
///
/// # Errors
/// Returns an error if the editor cannot be started or exits unsuccessfully.
pub(crate) fn open_config_file(template_path: &Path) -> Result<()> {
    let config_file_path = template_path.join("plato.toml");
    let (command, mut args) = get_default_editor()?;

    args.push(config_file_path.to_string_lossy().to_string());
    let mut child = Command::new(command).args(args).spawn()?;
    let status = child.wait()?;
    if !status.success() {
        bail!("Editor exited with non-zero exit code.")
    }

    Ok(())
}

pub(crate) fn setup_git(target: &Path) -> Result<()> {
    execute_command("git", ["init"], target)?;
    Ok(())
}

pub(crate) fn is_installed(cmd: &str) -> bool {
    Command::new(cmd).arg("--help").output().is_ok()
}

pub(crate) fn execute_command<I, S>(cmd: &str, args: I, target: &Path) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cmd_name = Path::new(cmd)
        .file_name()
        .and_then(|result| result.to_str())
        .unwrap_or(cmd);
    if !ALLOWED_CMD_RE.is_match(cmd_name) {
        bail!("Selected command '{cmd}' is not allowed");
    }
    let status = Command::new(cmd)
        .args(args)
        .current_dir(target)
        .status()
        .context(format!("Unable to run command {cmd}"))?;
    if !status.success() {
        bail!("Command {cmd} failed with status {status}");
    }
    Ok(())
}

pub(crate) fn bail_if_target_path_exists(target_path: &Path, force: bool) -> Result<()> {
    if !force && target_path.exists() {
        bail!(
            "Target path {} already exists. quitting.",
            &target_path.display()
        )
    }
    Ok(())
}

pub(crate) fn get_source_path_for_template(template_name: &str) -> Result<PathBuf> {
    let fallback_dirs = vec![];
    let global_plato_dir = get_global_plato_dir()?;
    let global_config = get_config(&global_plato_dir).ok();
    let extra_template_dirs = if let Some(config) = &global_config {
        &config.plato.extra_dirs
    } else {
        &fallback_dirs
    };
    let registry = TemplateRegistry::build(&global_plato_dir, extra_template_dirs);
    let (source_path, _) = registry.get(template_name)?;
    Ok(source_path.clone())
}
