pub(crate) mod args;
pub(crate) mod mapping;

use clap::Parser;
use plato::plugins::install::PluginInstallBackend;
use plato::{PluginInstallOptions, PluginRegisterOptions, RunOptions, ValidateOptions};

use self::args::{Cli, Commands, PluginCommands};
use self::mapping::{map_init_source_args, map_validate_source_args};

pub(crate) fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => {
            let (source, project_name) =
                map_init_source_args(args.template_name, args.project_name, args.path, args.git)?;

            plato::run(RunOptions {
                source,
                project_name,
                force: args.force,
                rev: args.rev,
                subpath: args.subpath,
                groups: args.groups,
                set_values: args.set_values,
                set_string_values: args.set_string_values,
            })
        }
        Commands::Val(args) => {
            let (source, project_name) = map_validate_source_args(
                args.template_name,
                args.project_name,
                args.path,
                args.git,
            )?;

            plato::validate(ValidateOptions {
                source,
                project_name,
                rev: args.rev,
                subpath: args.subpath,
                groups: args.groups,
                set_values: args.set_values,
                set_string_values: args.set_string_values,
            })
        }
        Commands::Config { template_name } => plato::edit_config(&template_name),
        Commands::List { verbose } => plato::display_templates(verbose),
        Commands::Plugin { command } => match command {
            PluginCommands::List => plato::display_plugins(),
            PluginCommands::Install(args) => {
                if args.git.is_some() && args.path.is_some() {
                    anyhow::bail!("--git and --path cannot be used together for plugin install");
                }
                let backend_flags = usize::from(args.cargo)
                    + usize::from(args.git.is_some())
                    + usize::from(args.uv_tool)
                    + usize::from(args.pipx);
                if backend_flags > 1 {
                    anyhow::bail!(
                        "Choose at most one plugin install backend: --cargo, --git, --uv-tool, or --pipx"
                    );
                }
                let backend = if let Some(path) = args.path {
                    if args.uv_tool {
                        PluginInstallBackend::UvToolPath { path }
                    } else if args.pipx {
                        PluginInstallBackend::PipxPath { path }
                    } else {
                        PluginInstallBackend::CargoPath { path }
                    }
                } else if let Some(url) = args.git {
                    PluginInstallBackend::Git { url }
                } else if args.uv_tool {
                    PluginInstallBackend::UvTool
                } else if args.pipx {
                    PluginInstallBackend::Pipx
                } else {
                    PluginInstallBackend::Cargo
                };
                plato::install_plugin(PluginInstallOptions {
                    name: args.name,
                    backend,
                })
            }
            PluginCommands::Register { name, command } => {
                plato::register_plugin(PluginRegisterOptions { name, command })
            }
            PluginCommands::Remove { name } => plato::remove_plugin(&name),
        },
    }
}
