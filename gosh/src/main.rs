mod abi;
mod blockchain;
mod commands;
mod config;
mod crypto;
mod env;
mod log;
mod profile;

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
        .subcommand(commands::build::command())
        .subcommand(commands::install::command())
        .subcommand_required(true)
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            commands::init::init_command().await?;
        }
        Some((commands::build::COMMAND, args)) => {
            commands::build::run(args).await?;
        }
        Some((commands::install::COMMAND, args)) => {
            commands::install::run(args).await?;
        }
        _ => {
            anyhow::bail!("Wrong subcommand");
        }
    }
    Ok(())
}
