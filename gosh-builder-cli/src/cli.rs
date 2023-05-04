use clap::Parser;
use std::path::PathBuf;

pub const DEFAULT_CONFIG_PATH: &str = "Gosh.yaml";

#[derive(Debug, Clone)]
pub struct CliSettings {
    pub config_path: PathBuf,
    pub workdir: PathBuf,
    pub validate: bool,
    // pub proxy_addr: SocketAddr,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "GOSH Builder Cli")]
struct BuilderCli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
    /// Validate image after build
    #[arg(long, default_value_t = false)]
    validate: bool,
    // #[arg(short, long)]
    // proxy_host: String,
    // #[arg(short, long)]
    // proxy_port: String,
}

pub fn settings() -> anyhow::Result<CliSettings> {
    let args = BuilderCli::parse();
    tracing::debug!("{:?}", args);

    let mut gosh_configfile = if let Some(ref raw_config_file) = args.config {
        PathBuf::from(raw_config_file)
    } else {
        PathBuf::from(DEFAULT_CONFIG_PATH)
    };

    if !gosh_configfile.exists() {
        panic!("Gosh config path doesn't exist");
    }
    gosh_configfile
        .canonicalize()
        .expect("gosh configfile path canonicalize");

    if !gosh_configfile.is_absolute() {
        gosh_configfile = std::env::current_dir()?.join(gosh_configfile);
    }

    let mut gosh_workdir = gosh_configfile.clone();
    gosh_workdir.pop();

    let cli_config = CliSettings {
        config_path: gosh_configfile,
        workdir: gosh_workdir,
        validate: args.validate,
    };

    Ok(cli_config)
}
