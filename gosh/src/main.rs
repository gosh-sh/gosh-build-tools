mod build;
mod config;
mod crypto;
mod ever_client;
mod init;

use clap::{Arg, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let version = option_env!("GOSH_BUILD_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    let matches = Command::new("gosh")
        .about("GOSH cli tool")
        .version(version)
        .subcommand(
            Command::new("init")
                .about("Start GOSH onboarding")
                .arg(Arg::new("GOSH_URL").required(false)),
        )
        .subcommand(Command::new("build").about("Run gosh-build-cli"))
        .get_matches();

    match matches.subcommand() {
        Some(("init", matches)) => {
            let gosh_url = matches.get_one::<String>("GOSH_URL");
            init::init_command(gosh_url).await?;
        }
        Some(("build", _)) => {
            build::build_command().await?;
        }
        _ => {
            anyhow::bail!("Wrong subcommand");
        }
    }
    Ok(())
}
