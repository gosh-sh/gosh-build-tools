use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CliSettings {
    pub config_path: PathBuf,
    pub workdir: PathBuf,
    // pub proxy_addr: SocketAddr,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
    // #[arg(short, long)]
    // proxy_host: String,
    // #[arg(short, long)]
    // proxy_port: String,
}

pub fn settings() -> anyhow::Result<CliSettings> {
    let args = Args::parse();
    tracing::debug!("Args {:?}", args);

    let mut gosh_configfile = PathBuf::from(args.config);
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
    };

    Ok(cli_config)
}
