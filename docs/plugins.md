# Plugin system

Plato setup plugins are external executables. A plugin named `uv` is a binary named:

```text
plato-plugin-uv
```

Plugins run after Plato renders and writes the project.

## Commands

A plugin should support:

```bash
plato-plugin-uv metadata
plato-plugin-uv setup
```

Protocol rules:

- stdout is JSON protocol output only.
- stderr is for logs, diagnostics, and child tool output.
- `setup` receives JSON on stdin.

## Metadata

`metadata` returns plugin identity and compatibility information:

```json
{
  "name": "uv",
  "version": "0.1.0",
  "supported_api_versions": [1],
  "capabilities": ["setup"]
}
```

## Setup

Plato sends a setup request containing the project root, step workdir, merged plugin config, template context, options, and environment metadata.

The plugin responds with:

```json
{
  "ok": true,
  "messages": ["uv setup complete"],
  "warnings": []
}
```

On failure, return `ok = false` with an error object, or exit non-zero with a useful stderr message.

## Discovery

Plato resolves plugins in this order:

1. explicit global registry entry
2. Plato-managed plugin directory
3. `PATH`

Managed plugins are installed under:

```text
$PLATO_HOME/plugins/bin
```

If `PLATO_HOME` is not set, Plato uses its global config directory.

## Plugin management

```bash
plato plugin list
plato plugin install uv
plato plugin install uv --path plugins/plato-plugin-uv
plato plugin install foo --git https://github.com/acme/plato-plugin-foo
plato plugin install plato-plugin-foo --uv-tool --path ./python-plugin
plato plugin install plato-plugin-foo --pipx --path ./python-plugin
plato plugin register foo --command /path/to/plato-plugin-foo
plato plugin remove foo
```

## First-party plugins

This repository includes first-party plugins:

- `plato-plugin-git`
- `plato-plugin-uv`
- `plato-plugin-pip`
- `plato-plugin-pnpm`
- `plato-plugin-cargo`
- `plato-plugin-precommit`

They live in `plugins/` as standalone packages and use the same external protocol as third-party plugins.

## Rust plugin authoring

Rust plugins can use:

- `plato-plugin-api` for protocol types
- `plato-plugin-support` for stdin/stdout runtime helpers and safe command execution

Minimal shape:

```rust
use plato_plugin_api::{PluginMetadata, PluginSetupRequest, PluginSetupResponse};
use plato_plugin_support::{SetupPlugin, run};

struct MyPlugin;

impl SetupPlugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata { /* ... */ }
    fn setup(&self, request: PluginSetupRequest) -> anyhow::Result<PluginSetupResponse> { /* ... */ }
}

fn main() -> std::process::ExitCode {
    run(MyPlugin)
}
```

Plugins can also be written in Python, Node, Go, shell, or any language that can read and write JSON.
