mod abi;
mod blockchain;
mod commands;
mod config;
mod crypto;
mod env;
mod profile;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gosh_utils::tracing::default_init();

    run().await.map_err(|err| {
        tracing::error!("{:?}", err);
        err
    })
}

async fn run() -> anyhow::Result<()> {
    let version = option_env!("GOSH_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    let matches = clap::Command::new("gosh")
        .about("\
GOSH cli tool https://gosh.sh

In case of any issues, or to receive assistance when working with GOSH cli please contact help@gosh.sh")
        .version(version)
        .subcommand(
            clap::Command::new("init")
                .about("Create a new GOSH Builder project in an existing directory"),
        )
        .subcommand(commands::anytree::command())
        .subcommand(commands::build::command())
        .subcommand(commands::install::command())
        .subcommand_required(true)
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => commands::init::init_command().await?,
        Some((commands::anytree::COMMAND, args)) => commands::anytree::run(args).await?,
        Some((commands::build::COMMAND, args)) => commands::build::run(args).await?,
        Some((commands::install::COMMAND, args)) => commands::install::run(args).await?,
        _ => anyhow::bail!("Wrong subcommand"),
    };
    Ok(())
}
