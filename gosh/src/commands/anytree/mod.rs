use super::{build, install};
use clap::ArgMatches;

pub const COMMAND: &str = "anytree";

pub fn command() -> clap::Command {
    clap::Command::new(COMMAND)
        .subcommand(build::command())
        .subcommand(install::command())
        .subcommand_required(true)
        .about("GOSH AnyTree")
}

pub async fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some((build::COMMAND, args)) => build::run(args).await,
        Some((install::COMMAND, args)) => install::run(args).await,
        _ => anyhow::bail!("Wrong subcommand"),
    }
}
