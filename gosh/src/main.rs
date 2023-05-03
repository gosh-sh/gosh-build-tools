mod commands;
mod config;
mod crypto;
mod ever_client;
mod log;

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
            clap::Command::new("build")
                .disable_help_flag(true)
                .disable_help_subcommand(true)
                .trailing_var_arg(true)
                .allow_hyphen_values(true)
                .about("Build GOSH Image")
                .arg(Arg::new("args").action(ArgAction::Set).num_args(..)),
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
