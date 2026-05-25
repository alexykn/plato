# Plato

Plato is a local project scaffolding tool. It renders configured templates into a new project directory and can optionally run language-specific setup for Python and Rust projects.

## CLI

```bash
plato init <template_name> <project_name> [--rev <rev>] [--subpath <path>] [--force]
plato init --git <git_spec> <project_name> [--rev <rev>] [--subpath <path>] [--force]
plato init --path <template_dir> <project_name> [--force]
plato config <template_name>
plato list [-v|--verbose]
```

Examples:

```bash
plato init py my-app
plato init api my-api --rev main
plato init --git gitlab:group/templates/api my-api
plato init --path ~/src/my-template demo
plato config api
plato list --verbose
```

## Global Configuration

Global app configuration lives at:

```text
~/.config/plato/config.toml
```

Template names are explicit. Directory names do not define templates.

```toml
[plato]
default_git_provider = "github"

[plato.git_hosts]
# optional provider-specific host overrides
# github = "github.company.test"
# gitlab = "gitlab.company.test"
# bitbucket = "bitbucket.company.test"

[templates]
py = { path = "~/.config/plato/templates/py" }
api = { git = "gitlab:platform/api-template", rev = "main" }
cli = { git = "github:owner/monorepo", subpath = "templates/cli" }

[template_configs]
api = "~/.config/plato/template_configs/api.toml"
py = "~/.config/plato/template_configs/py.toml"
```

### `[templates]`

Supported entries:

```toml
name = { path = "~/path/to/template" }
name = { git = "gitlab:group/repo", rev = "main", subpath = "templates/api" }
```

Configured Git templates do not require `--git`:

```bash
plato init api my-api
```

Ad-hoc Git templates do require `--git`:

```bash
plato init --git gitlab:group/repo my-api
```

### `[template_configs]`

Override configs apply to both path and Git templates. During `init`, an explicit override config wins over a template-local `plato.toml`; Plato prints a warning if both exist.

If no override exists and the template source has `plato.toml`, Plato uses the source config.

If neither exists, Plato uses default template behavior and prints a warning.

`plato config <name>` opens the override config if configured. If the override file does not exist yet, Plato creates it. If no override is configured, path templates can open their source `plato.toml`; remote templates fail with guidance to add `[template_configs]`.

Malformed configs remain editable with `plato config`.

## Git Templates

Supported Git specs in configured `{ git = ... }` entries or `--git`:

```text
github:owner/repo
gitlab:owner/group/repo
bitbucket:workspace/repo
host:owner/group/repo
git@host:owner/group/repo.git
ssh://git@host/owner/group/repo.git
https://host/owner/repo.git
```

Provider shorthand resolves to SSH remotes. For example:

```text
gitlab:owner/group/repo -> git@gitlab.com:owner/group/repo.git
```

`host:path` shorthand resolves to `git@host:path.git` and prints a warning that `git@` was added.

Revision precedence:

```text
--rev > inline #rev > configured rev
```

Subpath precedence:

```text
--subpath > configured subpath
```

URL fragments such as `repo.git#v1.2.0` are treated as Plato revision syntax.

### Git Authentication and Security

Plato does not manage credentials. Authentication is delegated to system Git, SSH agent, and Git credential helpers.

Plato rejects embedded credentials in Git URLs. Use SSH remotes or a configured Git credential helper instead.

## Template Configuration (`plato.toml`)

A template may contain `plato.toml`. This file is configuration only and is not copied into the generated project.

```toml
[plato]
template_language = "base"   # base | python | py | rust | rs
setup_git = false

[template.context]
# arbitrary key-value pairs for path and file content templates

[python]
language_version = "3"
package_manager = "auto"     # auto | uv | pip
project_scope = "auto"       # auto | base | requirements | install

[python.pip]
version_fallback = false

[rust]
toolchain = "stable"
project_scope = "auto"       # auto | base | fetch | build
project_type = "auto"        # auto | binary | bin | library | lib
cargo_init = false
```

## Rendering Rules

Path placeholders use `#key#` syntax:

```text
src/#project_name#/main.py.j2
```

File contents use MiniJinja:

```jinja2
{{ project_name }}
{{ language_version }}
{{ toolchain }}
```

Files ending in `.j2` or `.mj` are rendered and written without that extension. Non-template files are copied as raw bytes. `plato.toml` is never copied.

## List Output

```bash
plato list
plato list --verbose
```

`plato list` shows configured templates only and never performs network operations. Verbose mode also shows path/Git source details, rev, subpath, and override config paths.
