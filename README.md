# Plato

Plato is a local project scaffolding tool. It renders configured templates into a new project directory and can optionally run language-specific setup for Python and Rust projects.

## CLI

```bash
plato init <template_name> <project_name> [--rev <rev>] [--subpath <path>] [--force]
plato init --git <git_spec> <project_name> [--rev <rev>] [--subpath <path>] [--force]
plato init --path <template_dir> <project_name> [--force]
plato val <template_name> [project_name] [--rev <rev>] [--subpath <path>]
plato val --git <git_spec> [project_name] [--rev <rev>] [--subpath <path>]
plato val --path <template_dir> [project_name]
plato config <template_name>
plato list [-v|--verbose]
```

Examples:

```bash
plato init py my-app
plato init api my-api --rev main
plato init --git gitlab:group/templates/api my-api
plato init --path ~/src/my-template demo
plato val py
plato config api
plato list --verbose
```

## Global Configuration

Global app configuration lives at:

```text
~/.config/plato/config.toml
```

See `config.example.toml` for a complete global configuration example.

Template-local configuration lives in a template directory as `plato.toml`.
Complete examples are available in the repository root:

- `plato.base.example.toml`
- `plato.python.example.toml`
- `plato.rust.example.toml`

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

## Validation

`plato val` validates a template in memory. It resolves the same template sources as
`plato init`, renders the workspace, and applies path rewrites without creating
the target project directory. If no project name is provided, Plato renders with
`plato-validation`.

It does not run setup tools such as `uv`, `pip`, `cargo`, or `git init`, so it
does not prove dependency installation succeeds. It is intended to catch Plato
template/config errors, such as invalid `plato.toml`, invalid path rewrite rules,
template syntax errors, duplicate rendered paths, and references to undefined
template variables.

## Template Configuration (`plato.toml`)

A template may contain `plato.toml`. This file is configuration only and is not copied into the generated project.

```toml
[plato]
template_language = "base"   # base | python | py | rust | rs
setup_git = false

[git]
# If unset, Plato runs plain `git init` and Git uses the user's global defaults.
# initial_branch = "main"

[git.user]
# name = "Jane Doe"
# email = "jane@example.com"
# signing_key = "ABC123DEF456"

[git.commit]
# gpgsign = true
initial = false
initial_message = "Initial commit"

[git.core]
# hooks_path = ".githooks"
# autocrlf = false          # true | false | input
# eol = "lf"               # lf | crlf | native
# filemode = true

[template.context]
# arbitrary key-value pairs for path and file content templates

[path.replace]
# Named path rewrite rules. `path` must exactly match a relative path in the
# template source tree. `replace` is rendered with MiniJinja before writing.
# package = { path = "src/package-template", replace = "src/{{ project_snake }}" }

[python]
language_version = "3"
package_manager = "auto"     # auto | uv | pip
project_scope = "auto"       # auto | base | requirements | install

[python.install]
# Explicit setup-time install selectors. Plato does not create or infer these.
groups = []
extras = []

[python.pip]
version_fallback = false

[rust]
toolchain = "stable"
project_scope = "auto"       # auto | base | fetch | build
project_type = "auto"        # auto | binary | bin | library | lib
cargo_init = false
```

`[python.install]` applies only when the resolved Python project scope is
`install` or `requirements`. Plato passes only explicitly configured selectors;
it does not create or infer dependency groups/extras.

| Resolved setup path | `extras` | `groups` |
| --- | --- | --- |
| `uv sync` | yes | yes |
| `uv sync --no-install-project` | yes | yes |
| editable install, e.g. `pip install -e ...` or `uv pip install -e ...` | yes | no |
| requirements file install, e.g. `pip install -r ...` or `uv pip install -r ...` | no | no |

`groups` require a modern `pyproject.toml` with a `[project]` table and the `uv`
package manager, so Plato can install them through project metadata instead of
guessing from template files. `extras` are passed to editable installs as
targets such as `.[cli]`.

Plato does not validate Python package metadata. `uv`, `pip`, and the selected
build backend remain responsible for errors such as missing extras, missing
dependency groups, invalid package layout, or missing README files. Plato still
does not infer layouts, create dependency groups/extras, or rewrite package paths
unless the template explicitly asks for that through rendering or
`[path.replace]`.

## Rendering Rules

Plato provides derived project-name values to path and file content templates.
These values are context only: Plato never renames paths or rewrites file
contents unless the template explicitly references them in `[path.replace]` or
in a `.j2`/`.mj` file.

Given:

```bash
plato init py my-cool-app
```

all templates receive:

```jinja2
{{ project_name }}   {# my-cool-app, exactly as passed on the CLI #}
{{ project_kebab }}  {# my-cool-app #}
{{ project_snake }}  {# my_cool_app #}
{{ project_pascal }} {# MyCoolApp #}
```

Python templates also receive:

```jinja2
{{ python_distribution_name }} {# my-cool-app #}
{{ python_package_name }}      {# my_cool_app #}
{{ python_cli_name }}          {# my-cool-app #}
{{ language_version }}
```

Rust templates also receive:

```jinja2
{{ rust_package_name }} {# my-cool-app #}
{{ rust_crate_name }}   {# my_cool_app #}
{{ rust_binary_name }}  {# my-cool-app #}
{{ toolchain }}
```

Values in `[template.context]` override built-in context values with the same
key.

Template source paths should stay regular, navigable filesystem paths. When a
path needs to change in the generated project, define a named rewrite rule in
`plato.toml`:

```toml
[path.replace]
source = { path = "src/package-template", replace = "src/{{ python_package_name }}" }
```

The `path` value must exactly match a relative file or directory path in the
template root. Directory rewrites apply to the whole subtree, so
`src/package-template/__init__.py.j2` becomes
`src/my_cool_app/__init__.py` when the project name is `my-cool-app` in the
example above. Replacements are rendered with the same MiniJinja context and
filters as file contents.

File contents use MiniJinja:

```jinja2
{{ project_name }}
{{ project_snake }}
{{ python_package_name }}
{{ language_version }}
{{ toolchain }}
```

Plato also adds Ansible-style regex filters to MiniJinja templates:

```jinja2
{{ value | regex_replace('^py3-', '') }}
{{ value | regex_search('\\d+') }}
{{ value | regex_findall('\\d+') }}
{{ value | regex_escape }}
```

Capture replacements use Ansible/Python-style syntax:

```jinja2
{{ 'py3-requests' | regex_replace('^py3-(.*)$', '\\1') }}
{{ 'pkg:requests' | regex_replace('^(?P<kind>[^:]+):(?P<name>.+)$', '\\g<name>') }}
```

Regex filters are available in `.j2` and `.mj` file contents and in `[path.replace]` replacement strings. Plato uses Rust regex syntax, so common regexes work, but Python `re` features such as look-around and backreferences inside patterns are not supported.

Files ending in `.j2` or `.mj` are rendered and written without that extension. Non-template files are copied as raw bytes. `plato.toml` is never copied.

## List Output

```bash
plato list
plato list --verbose
```

`plato list` shows configured templates only and never performs network operations. Verbose mode also shows path/Git source details, rev, subpath, and override config paths.
