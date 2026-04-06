use anyhow::{Context, Result, bail};
use regex::Regex;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

static ALLOWED_CMD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(git|cargo|uv|python\d*(?:\.\d+)*)$").expect("Invalid regex pattern")
});

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProjectScope {
    Requirements,
    Install,
    Base,
}

pub(crate) fn setup_git(target: &Path) -> Result<()> {
    execute_command("git", &["init"], target)?;
    Ok(())
}

pub(crate) fn is_installed(cmd: &str) -> bool {
    Command::new(cmd).arg("--help").output().is_ok()
}

pub(crate) fn execute_command(cmd: &str, args: &[&str], target: &Path) -> Result<()> {
    let cmd_name = Path::new(cmd)
        .file_name()
        .and_then(|result| result.to_str())
        .unwrap_or(cmd);
    if !ALLOWED_CMD_RE.is_match(cmd_name) {
        bail!("Selected command '{cmd}' is not allowed");
    }
    Command::new(cmd)
        .args(args)
        .current_dir(target)
        .status()
        .context(format!("Unable to run command {cmd}"))?;
    Ok(())
}
