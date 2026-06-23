use anyhow::{Context, Result, anyhow, bail};
use plato_plugin_api::{
    PLUGIN_API_VERSION, PluginMetadata, PluginSetupRequest, PluginSetupResponse,
};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn read_metadata(command: &Path) -> Result<PluginMetadata> {
    let output = Command::new(command)
        .arg("metadata")
        .output()
        .with_context(|| {
            format!(
                "Failed to run plugin metadata command {}",
                command.display()
            )
        })?;
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if !output.status.success() {
        bail!("Plugin metadata command failed: {}", command.display());
    }
    let metadata = serde_json::from_slice::<PluginMetadata>(&output.stdout).with_context(|| {
        format!(
            "Plugin {} returned invalid metadata JSON",
            command.display()
        )
    })?;
    if !metadata
        .supported_api_versions
        .contains(&PLUGIN_API_VERSION)
    {
        bail!(
            "Plugin {} does not support Plato plugin API v{}",
            metadata.name,
            PLUGIN_API_VERSION
        );
    }
    Ok(metadata)
}

pub(crate) fn run_setup(
    command: &Path,
    request: &PluginSetupRequest,
) -> Result<PluginSetupResponse> {
    let mut child = Command::new(command)
        .arg("setup")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("Failed to run plugin setup command {}", command.display()))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Plugin stdin was unavailable"))?;
        serde_json::to_writer(&mut *stdin, request)?;
        stdin.write_all(b"\n")?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!("Plugin setup command failed: {}", command.display());
    }

    let response =
        serde_json::from_slice::<PluginSetupResponse>(&output.stdout).with_context(|| {
            format!(
                "Plugin {} returned invalid setup response JSON",
                command.display()
            )
        })?;
    if !response.ok {
        if let Some(error) = &response.error {
            bail!("Plugin {} failed: {}", request.plugin, error.message);
        }
        bail!("Plugin {} failed without an error message", request.plugin);
    }
    Ok(response)
}
