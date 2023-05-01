mod abi;
mod blockchain;
mod build;
mod config;
mod crypto;
mod env;
mod init;
mod profile;

use clap::Command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let version = option_env!("GOSH_BUILD_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    let matches = Command::new("gosh")
        .about("GOSH cli tool")
        .version(version)
        .subcommand(Command::new("init").about("Start GOSH onboarding"))
        .subcommand(Command::new("build").about("Run gosh-build-cli"))
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            init::init_command().await?;
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