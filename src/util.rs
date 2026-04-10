use anyhow::{Context, Result, bail};
use regex::Regex;
use std::env::var;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

static ALLOWED_CMD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(git|cargo|uv|python\d*(?:\.\d+)*)$").expect("Invalid regex pattern")
});

fn get_default_editor() -> OsString {
    if let Ok(visual) = var("VISUAL")
        && !visual.trim().is_empty()
    {
        return visual.into();
    }
    if let Ok(editor) = var("EDITOR")
        && !editor.trim().is_empty()
    {
        return editor.into();
    }
    "nano".into()
}

/// Opens the template's `plato.toml` in the user's editor.
///
/// # Errors
/// Returns an error if the editor cannot be started or exits unsuccessfully.
pub fn open_config_file(template_path: &Path) -> Result<()> {
    let config_file_path = template_path.join("plato.toml");
    let editor = get_default_editor();

    let mut child = Command::new(editor).arg(config_file_path).spawn()?;
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
