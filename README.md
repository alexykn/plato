# Plato

Plato is a local project scaffolding tool. It renders configured templates into a new project directory and can optionally run language-specific setup for Python and Rust projects.

## CLI

```bash
plato init <template_name> <project_name> [--rev <rev>] [--subpath <path>] [--force] [-g <group>] [-s <key=value>] [--set-string <key=value>]
plato init --git <git_spec> <project_name> [--rev <rev>] [--subpath <path>] [--force] [-g <group>] [-s <key=value>] [--set-string <key=value>]
plato init --path <template_dir> <project_name> [--force] [-g <group>] [-s <key=value>] [--set-string <key=value>]
plato val <template_name> [project_name] [--rev <rev>] [--subpath <path>] [-g <group>] [-s <key=value>] [--set-string <key=value>]
plato val --git <git_spec> [project_name] [--rev <rev>] [--subpath <path>] [-g <group>] [-s <key=value>] [--set-string <key=value>]
plato val --path <template_dir> [project_name] [-g <group>] [-s <key=value>] [--set-string <key=value>]
plato config <template_name>
plato list [-v|--verbose]
```

Examples:

```bash
plato init py my-app
plato init api my-api --rev main
plato init api my-api -g docker -s port=8000
plato init --git gitlab:group/templates/api my-api
plato init --path ~/src/my-template demo
plato val py -g docker -s docker=true
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

Plato validates Plato mechanics only. It does not validate Python or Rust package
metadata, workspace layout correctness, or whether `uv`, `pip`, or `cargo` setup
will succeed.

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
# arbitrary typed values for path and file content templates
# strings, booleans, numbers, arrays, and tables are supported
# docker = false
# port = 8000
# features = ["api", "metrics"]

[path.replace]
# Named path rewrite rules. `path` must exactly match a relative path in the
# template source tree. `replace` is rendered with MiniJinja before writing.
# package = { path = "src/package-template", replace = "src/{{ project_snake }}" }

[path.exclude]
# Named path exclusion rules. Missing paths are ignored.
# dockerfile = { path = "Dockerfile", unless = "docker" }
# secret = { path = ".secret" }

[python]
language_version = "3"
package_manager = "uv"       # uv | pip
project_scope = "base"       # base | requirements | install

[python.uv]
setup = "editable"           # editable | sync

[python.install]
# Explicit setup-time install selectors. Plato does not create or infer these.
groups = []
extras = []

[rust]
toolchain = "stable"
components = []             # e.g. ["rustfmt", "clippy"]
targets = []                # e.g. ["wasm32-unknown-unknown"]
project_scope = "base"       # base | fetch | build
project_type = "binary"      # binary | bin | library | lib
cargo_init = false
```

`[python.install]` applies only when the resolved Python project scope is
`install` or `requirements`. Plato passes only explicitly configured selectors;
it does not create or infer dependency groups/extras.

| Package manager/setup | Scope | Command shape | `extras` | `groups` |
| --- | --- | --- | --- | --- |
| `uv`, `setup = "editable"` | `install` | `uv pip install -e ...` | yes | no |
| `uv`, `setup = "editable"` | `requirements` | `uv pip install -r ...` | no | no |
| `uv`, `setup = "sync"` | `install` | `uv sync` | yes | yes |
| `uv`, `setup = "sync"` | `requirements` | `uv sync --no-install-project` | yes | yes |
| `pip` | `install` | `pip install -e ...` | yes | no |
| `pip` | `requirements` | `pip install -r ...` | no | no |

`[python.uv].setup = "editable"` is the default when omitted. `sync` is
available only with `package_manager = "uv"`. `groups` are passed only to
`uv sync`; `extras` are passed to `uv sync` and editable installs as targets such
as `.[cli]`.

Plato does not validate Python package metadata. `uv`, `pip`, and the selected
build backend remain responsible for errors such as missing extras, missing
dependency groups, invalid package layout, or missing README files. Plato still
does not infer layouts, create dependency groups/extras, or rewrite package paths
unless the template explicitly asks for that through rendering or
`[path.replace]`.

Rust setup is explicit too. Plato installs the configured toolchain with
`rustup`, adds configured components/targets, and then runs only the selected
Cargo setup scope.

| Scope | Command shape |
| --- | --- |
| `base` | `rustup toolchain install <toolchain>` plus configured components/targets |
| `fetch` | base setup + `cargo +<toolchain> fetch` |
| `build` | base setup + `cargo +<toolchain> build` |

When `cargo_init = true`, Plato runs `cargo +<toolchain> init --bin` or
`cargo +<toolchain> init --lib` before the selected scope. Leave it `false` when
the template owns `Cargo.toml` and the source layout. Plato does not generate
`rust-toolchain.toml`; include it in the template if the project should commit
one.

## Context Overrides and Groups

Templates can receive typed context values from built-ins, `plato.toml`, selected
groups, and CLI overrides. Precedence is:

```text
built-ins < [template.context] < selected groups in CLI order < -s/--set/--set-string
```

`-s`/`--set` infers scalar types and arrays:

```bash
plato init api my-api -s docker=true -s port=8000 -s 'features=[api,metrics]'
plato init api my-api --set-string version=1.0
```

Inference rules:

```text
1000       -> integer
100.0      -> float
true/False -> boolean
[a,1,true] -> array with inferred items
other      -> string
```

Dotted keys create nested objects:

```bash
plato init api my-api -s author.name=Jane -s author.email=jane@example.com
```

Optional template groups use reserved root-level files named
`plato.<group>.toml`:

```text
plato.toml
plato.docker.toml
plato.ci.toml
```

Select them with `-g`/`--group`:

```bash
plato init api my-api -g docker -g ci
```

Group files support only rendering/path-selection config:

```toml
# plato.docker.toml

[template.context]
docker = true
docker_image = "api"

[path.replace]
dockerfile = { path = "Dockerfile.template", replace = "Dockerfile" }

[path.exclude]
compose = { path = "compose.dev.yml", unless = "docker" }
```

Groups may set `[template.context]`, `[path.replace]`, and `[path.exclude]`.
They cannot change `[plato]`, `[python]`, `[rust]`, or `[git]` setup.
`plato.toml` and root-level `plato.*.toml` files are never copied or rendered
into the generated project.

`[path.exclude]` removes files or directories from the in-memory workspace before
path rewrites and rendering. Missing exclude paths are ignored. `unless` names a
top-level context variable; the path is excluded unless that value is truthy.

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
