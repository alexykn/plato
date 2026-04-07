# Plato

Plato is a local project scaffolding tool.

I built this because I wanted a "one-shot" command to setup my projects in a state where I can just start coding.

It reads templates from `~/.config/plato/<template_name>`, renders files into a new
project directory, and then optionally performs language-specific setup for Python
and Rust projects.

Current CLI:

```bash
plato init <template_name> <project_name>
plato config <template_name>
plato list
```

Examples:

```bash
plato init py my-app
plato init rs-bin hello-rust
plato config py
plato list
```

---

## Project Structure & Module Organization
- `src/main.rs`: CLI entrypoint. Parses `plato init <template> <project_name>`, `plato config <template>`, and `plato list`.
- `src/lib.rs`: Main orchestration entrypoint for rendering + language setup.
- `src/core/config.rs`: `plato.toml` schema, defaults, and config loading.
- `src/core/guard.rs`: Cleanup guard that removes the target directory on failure.
- `src/workspace/mod.rs`: Builds render context and drives template rendering.
- `src/workspace/setup.rs`: Scans source templates, renders paths/content, writes files.
- `src/languages/mod.rs`: Shared setup context and language dispatch.
- `src/languages/python/*`: Python detection and setup via `uv` or `pip`.
- `src/languages/rust/*`: Rust detection and setup via `cargo`.
- `src/util.rs`: Command allowlist, command execution, and tool detection.

Template source location:
- `~/.config/plato/<template_name>/plato.toml`
- plus any files/directories that belong to the template

---

## Architecture & Setup Flow
Execution flow:

1. CLI parses either `template_name` + `project_name` for `init`, or just `template_name` for `config`
2. Plato resolves the template source directory at `~/.config/plato/<template_name>`
3. For `plato config`, Plato opens that template’s `plato.toml` in your editor
4. Plato loads `plato.toml`
4. Plato creates the target directory `./<project_name>`
5. Plato scans the template directory
6. Plato renders:
   - file paths using `#...#` placeholders
   - file contents using MiniJinja (`{{ ... }}`)
7. Plato optionally runs language-specific setup:
   - Python: venv + dependency sync/install
   - Rust: optional `cargo init`, then `cargo fetch` or `cargo build`
8. Plato optionally runs `git init`
9. If setup fails at any point, the target directory is cleaned up automatically

Important behavior:
- `plato.toml` is configuration only and is **not copied** into the generated project.
- Files ending in `.j2` or `.mj` are rendered as templates and then written without that extension.
- Non-template files are copied as raw bytes.
- Empty templates are supported.

---

## Template Rendering Rules

### Path placeholders
Path rendering uses `#key#` placeholders.

Supported path keys include:
- built-in keys:
  - `#project_name#`
  - `#language_version#` for Python templates
  - `#toolchain#` for Rust templates
- custom keys from `[template.context]`

Example path:

```text
src/#project_name#/__init__.py.j2
```

for project name `demo` becomes:

```text
src/demo/__init__.py
```

### File content placeholders
File contents use MiniJinja syntax:

```jinja2
{{ project_name }}
{{ language_version }}
{{ toolchain }}
```

Custom values from `[template.context]` are available in both path rendering and file content templates.

Example:

```toml
[template.context]
author = "Alice"
license = "MIT"
```

```jinja2
# {{ project_name }}
Author: {{ author }}
License: {{ license }}
```

Example custom path usage:

```toml
[template.context]
author = "alice"
```

A path like:

```text
docs/#author#/intro.md.j2
```

becomes:

```text
docs/alice/intro.md
```

---

## Configuration Reference (`plato.toml`)

Each template must contain a `plato.toml` file.

Full schema:

```toml
[plato]
template_language = "base"   # base | python | py | rust | rs
setup_git = false            # default: false

[template.context]
# arbitrary key-value pairs for path and file content templates
# example:
# author = "Alice"

[python]
language_version = "3"      # default: "3"
package_manager = "auto"    # auto | uv | pip
project_scope = "auto"      # auto | base | requirements | install

[rust]
toolchain = "stable"        # default: "stable"
project_scope = "auto"      # auto | base | fetch | build
project_type = "auto"       # auto | binary | bin | library | lib
cargo_init = false           # default: false
```

### `[plato]`
- `template_language`
  - `base`: render files only, no language setup
  - `python` / `py`: enable Python setup logic
  - `rust` / `rs`: enable Rust setup logic
- `setup_git`
  - if `true`, Plato runs `git init` in the generated target
  - default is `false`

### `plato config`
- `plato config <template_name>` opens the template’s `plato.toml` in your editor
- it prefers `$VISUAL`, then `$EDITOR`, then falls back to `nano`

### `plato list`
- `plato list` prints the available template directories from `~/.config/plato`

### `[template.context]`
Arbitrary string key-value pairs for path and content templates.

### `[python]`
- `language_version`
  - passed to Python setup tools
  - used in template context as `language_version`
  - default: `"3"`
- `package_manager`
  - `auto`: prefer `uv` if installed, else `python<version>` + `pip`
  - `uv`: force `uv`
  - `pip`: force `pip`
- `project_scope`
  - `auto`: infer from rendered project files
  - `base`: create no Python environment or dependency install
  - `requirements`: install dependencies only
  - `install`: install the project itself

### `[rust]`
- `toolchain`
  - available in template content as `toolchain`
  - default: `"stable"`
- `project_scope`
  - `auto`: infer from rendered project files
  - `base`: no Cargo command after rendering
  - `fetch`: run `cargo fetch`
  - `build`: run `cargo build`
- `project_type`
  - `auto`: infer binary vs library from the rendered target
  - `binary` / `bin`: binary project
  - `library` / `lib`: library project
- `cargo_init`
  - `true`: run `cargo init --bin` or `cargo init --lib`
  - `false`: skip `cargo init`
  - default: `false`

---

## Python Setup Behavior

### Package manager detection
Python package manager auto-detection currently works like this:
1. if `uv` is installed, use `uv`
2. else if `python<language_version>` is installed, use `pip`
3. else no Python setup is performed

### Python scope detection
Python `project_scope = "auto"` resolves as follows:
- `install`
  - when both of these exist in the rendered target:
    - `pyproject.toml`
    - `src/<project_name>/__init__.py`
- `requirements`
  - when `pyproject.toml` exists, or `requirements.txt` exists
- `base`
  - otherwise

### Python setup commands
If using `uv`:
- always create `.venv` with `uv venv --python <version>`
- `install` → `uv sync`
- `requirements` → `uv sync --no-install-project`
- `base` → do nothing after rendering

If using `pip`:
- always create `.venv` with `python<version> -m venv .venv`
- `install` → `python -m pip install -e .` plus detected dependency groups
- `requirements` → install from `requirements.txt`
- if `requirements.txt` is missing but `pyproject.toml` exists, Plato may generate
  `.plato/requirements.txt` from `project.dependencies`, dependency groups, and
  legacy `tool.uv.dev-dependencies`

Additional Python behavior:
- Plato may generate a minimal `README.md` when needed for package installation
- current Python support is focused on modern `pyproject.toml` projects and
  `requirements.txt` workflows

### Python template example: modern uv project

Template tree:

```text
~/.config/plato/py/
├── plato.toml
├── pyproject.toml.j2
├── README.md.j2
└── src/
    └── #project_name#/
        ├── __init__.py
        └── main.py.j2
```

`plato.toml`:

```toml
[plato]
template_language = "python"
setup_git = true

[python]
language_version = "3.12"
package_manager = "uv"
project_scope = "install"

[template.context]
author = "Alice"
```

`pyproject.toml.j2`:

```toml
[project]
name = "{{ project_name }}"
version = "0.1.0"
description = "Generated by Plato"
readme = "README.md"
requires-python = ">={{ language_version }}"
dependencies = []

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
```

`src/#project_name#/main.py.j2`:

```python
from . import __name__


def main() -> None:
    print(f"hello from {__name__}")
```

### Python template example: requirements-only project

```toml
[plato]
template_language = "python"
setup_git = false

[python]
language_version = "3.11"
package_manager = "auto"
project_scope = "requirements"
```

A matching template may include only:
- `requirements.txt`
- application files
- no installable package metadata

---

## Rust Setup Behavior

### Rust package manager detection
Rust currently supports Cargo only:
- if `cargo` is installed, use Cargo setup
- otherwise no Rust setup is performed

### Rust scope detection
Rust `project_scope = "auto"` resolves as follows:
- `base`
  - if `Cargo.toml` does not exist in the rendered target
- `build`
  - if `Cargo.toml` exists and one of the following exists:
    - `src/main.rs`
    - `src/lib.rs`
    - at least one `*.rs` file inside `src/bin/`
- `fetch`
  - if `Cargo.toml` exists but no buildable Rust target was detected

### Rust type detection
Rust `project_type = "auto"` resolves as follows:
- if `Cargo.toml` does not exist, default to `binary`
- otherwise prefer `binary` if binary markers are found
- otherwise use `library` if library markers are found
- otherwise default to `binary`

Binary markers:
- `src/main.rs`
- `src/bin/`
- `[[bin]]` in `Cargo.toml`

Library markers:
- `src/lib.rs`
- `[lib]` in `Cargo.toml`

### Rust setup commands
If `cargo_init = true`:
- `binary` → `cargo init --bin --vcs none`
- `library` → `cargo init --lib --vcs none`

Then scope is applied:
- `build` → `cargo build`
- `fetch` → `cargo fetch`
- `base` → no Cargo command

Important Rust rule:
- if `cargo_init = false`, Plato assumes your template already contains a valid
  Cargo project when you ask it to `fetch` or `build`
- if `cargo_init = false` and there is no `Cargo.toml`, then auto scope becomes
  `base` and Plato will only render files

### Rust template example: empty scaffold via Cargo
Use this when you want Cargo to create the package files.

Template tree:

```text
~/.config/plato/rs-bin/
└── plato.toml
```

`plato.toml`:

```toml
[plato]
template_language = "rust"
setup_git = false

[rust]
project_type = "binary"
project_scope = "build"
cargo_init = true
```

Result:
- Plato creates the target directory
- `cargo init --bin --vcs none`
- `cargo build`

### Rust template example: existing Cargo template
Use this when your template already includes `Cargo.toml` and source files.

Template tree:

```text
~/.config/plato/rs-lib/
├── plato.toml
├── Cargo.toml.j2
├── README.md.j2
└── src/
    └── lib.rs.j2
```

`plato.toml`:

```toml
[plato]
template_language = "rust"
setup_git = true

[rust]
project_type = "library"
project_scope = "build"
cargo_init = false
```

`Cargo.toml.j2`:

```toml
[package]
name = "{{ project_name }}"
version = "0.1.0"
edition = "2024"

[dependencies]
```

`src/lib.rs.j2`:

```rust
pub fn greet() -> &'static str {
    "hello from {{ project_name }}"
}
```

### Rust template example: template-only render
This is valid when you intentionally want rendered Rust files but no cargo init setup.

```toml
[plato]
template_language = "rust"
setup_git = false

[rust]
project_scope = "auto"
project_type = "auto"
cargo_init = false
```

If the rendered target has no `Cargo.toml`, auto scope becomes `base` and Plato
just writes the files.

---

## Base Template Example

Base templates only render files and optionally initialize git.

Template tree:

```text
~/.config/plato/base-docs/
├── plato.toml
├── README.md.j2
└── docs/
    └── intro.md.j2
```

`plato.toml`:

```toml
[plato]
template_language = "base"
setup_git = true

[template.context]
author = "Alice"
team = "Platform"
```

`README.md.j2`:

```md
# {{ project_name }}

Created by {{ author }} from the {{ team }} team.
```

---

## Supported Template Files
- `plato.toml`: required, config only, never copied to target
- `*.j2`, `*.mj`: rendered as MiniJinja templates, extension stripped
- all other files: copied as-is
- directories: reproduced in the target

---

## Current Limitations
- Python auto-detection currently focuses on `pyproject.toml` and `requirements.txt`
- legacy Python packaging like `setup.py` is not yet part of the current setup path
- Rust currently supports Cargo only
- Rust auto-detection handles binary vs library, but is intentionally conservative
- path templating uses `#key#` placeholders, while file contents use MiniJinja `{{ key }}` syntax
