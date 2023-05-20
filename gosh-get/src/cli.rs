use clap::{Parser, Subcommand};

const DEFAULT_GOSH_HTTP_PROXY: &str = "127.0.0.1:6054";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// GOSH http proxy address
    #[arg(short, long, env = "GOSH_HTTP_PROXY", default_value = DEFAULT_GOSH_HTTP_PROXY, value_name = "HOST:PORT")]
    pub proxy_addr: String,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Get commit state as a file tree without git history
    Commit { gosh_url: String, commit: String },
    /// Get the single file from specific commit
    File {
        gosh_url: String,
        commit: String,
        path: String,
    },
}
