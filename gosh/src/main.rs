mod config;
mod crypto;
mod init;
mod abi;
mod blockchain;
mod env;

use clap::Command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let version = option_env!("GOSH_BUILD_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    let matches = Command::new("gosh")
        .about("GOSH cli tool")
        .version(version)
        .subcommand(Command::new("init").about("Start GOSH onboarding"))
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            init::init_command()?;
        }
        _ => {
            anyhow::bail!("Wrong subcommand");
        }
    }
    Ok(())
}
