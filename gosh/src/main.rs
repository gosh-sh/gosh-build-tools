mod abi;
mod blockchain;
mod commands;
mod config;
mod crypto;
mod env;
mod log;
mod profile;

use clap::{Arg, ArgAction};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::init();

    let version = option_env!("GOSH_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    let matches = clap::Command::new("gosh")
        .about("GOSH cli tool https://gosh.sh")
        .version(version)
        .subcommand(
            clap::Command::new("init")
                .about("Create a new GOSH Builder project in an existing directory"),
        )
        .subcommand(
            // TODO: merge gosh-builder-cli and gosh
            gosh_builder::cli::cli_command(),
        )
        .subcommand(
            clap::Command::new("pull")
                .arg(
                    Arg::new("install")
                        .short('i')
                        .long("install")
                        .action(ArgAction::SetTrue),
                )
                .arg(Arg::new("gosh_url"))
                .about("Pull GOSH repo"),
        )
        .subcommand(
            clap::Command::new("install")
                .arg(Arg::new("gosh_url"))
                .about("Install GOSH repo"),
        )
        .subcommand_required(true)
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            commands::init::init_command().await?;
        }
        Some(("build", args)) => {
            commands::build::build_command(args).await?;
        }
        _ => {
            anyhow::bail!("Wrong subcommand");
        }
    }
    Ok(())
}
