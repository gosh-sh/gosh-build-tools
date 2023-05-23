use std::{net::SocketAddr, path::PathBuf};

use clap::ArgMatches;
use gosh_builder::docker_builder::git_context::GitContext;

pub const COMMAND: &str = "build";
pub const DEFAULT_CONFIG_PATH: &str = "Gosh.yaml";
pub const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:6054";

#[derive(Debug, Clone)]
pub struct CliSettings {
    pub config_path: PathBuf,
    pub workdir: PathBuf,
    pub validate: bool,
    pub quiet: bool,
    pub git_context: Option<GitContext>,
    pub sbom_proxy_socket: SocketAddr,
}

pub async fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    gosh_builder::cli::run(matches).await
}

pub fn command() -> clap::Command {
    // IMPORTANT
    // [ArgAction::Count] instead of [ArgAction::SetTrue] is intentional
    // because `--quiet` and other flags can be used multiple times (like in `docker build`)
    // and due to the chain of calls (e.g. telepresence -> bash_shortcut -> docker build)
    // some important flags might appear multiple times
    clap::Command::new(COMMAND)
        .about("Build GOSH image from `--config` or from [url]")
        .arg(
            clap::Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(clap::ArgAction::Count)
                .help("Suppress output"),
        )
        .arg(
            clap::Arg::new("validate")
                .long("validate")
                .action(clap::ArgAction::Count)
                .help("Validate the result image"),
        )
        .arg(
            clap::Arg::new("socket")
                .short('s')
                .long("socket")
                .help("Socket address for the SBOM proxy server")
                .value_name("IP:PORT")
                .default_value(DEFAULT_SOCKET_ADDR),
        )
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .value_name("PATH")
                .help("Config path (in case of GOSH url context it should be relative to the root)")
                .default_value(DEFAULT_CONFIG_PATH),
        )
        .arg(
            clap::Arg::new("url")
                .value_name("gosh://0:...")
                .required(false),
        )
}
