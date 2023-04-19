use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Commit {
        gosh_url: String,
        commit: String,
    },
    File {
        gosh_url: String,
        commit: String,
        path: String,
    },
    // TODO: add optional <dest> like it works in git
}

pub fn init() -> anyhow::Result<Cli> {
    let cli = Cli::parse();
    Ok(cli)
}
