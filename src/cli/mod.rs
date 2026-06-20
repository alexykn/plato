pub(crate) mod args;
pub(crate) mod mapping;

use clap::Parser;
use plato::{RunOptions, ValidateOptions};

use self::args::{Cli, Commands};
use self::mapping::map_template_source_args;

pub(crate) fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => {
            let (source, project_name) = map_template_source_args(
                args.template_name,
                args.project_name,
                args.path,
                args.git,
            )?;

            plato::run(RunOptions {
                source,
                project_name,
                force: args.force,
                rev: args.rev,
                subpath: args.subpath,
            })
        }
        Commands::Val(args) => {
            let (source, project_name) = map_template_source_args(
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
            })
        }
        Commands::Config { template_name } => plato::edit_config(&template_name),
        Commands::List { verbose } => plato::display_templates(verbose),
    }
}
