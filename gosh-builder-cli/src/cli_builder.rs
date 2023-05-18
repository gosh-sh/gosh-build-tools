use crate::{
    cli::{CliSettings, DEFAULT_CONFIG_PATH},
    docker_builder::{GoshBuilder, ImageBuilder},
    git_cache::registry::GitCacheRegistry,
    grpc_server,
    sbom::{self, Sbom},
};
use clap::ArgMatches;
use gosh_builder_config::GoshConfig;
use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::Mutex;

pub fn cli_command() -> clap::Command {
    clap::Command::new("build")
        .about("Build GOSH image from `--config` or from [url]")
        .arg(clap::arg!(-q --quiet "Suppress output"))
        .arg(clap::arg!(--validate "Validate the result image"))
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .value_name("PATH")
                .help("Config path (in case of GOSH url context it should be relative to the root)")
                .default_value(DEFAULT_CONFIG_PATH),
        )
        .arg(clap::Arg::new("url").required(false))
}

pub fn settings(matches: &ArgMatches) -> anyhow::Result<CliSettings> {
    let mut gosh_configfile = PathBuf::from(
        matches
            .get_one::<String>("config")
            .expect("should never fail due to `.default_value`"),
    );

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

    let context = match matches.try_get_one::<String>("url")? {
        Some(gosh_url) => Some(gosh_url.parse()?),
        None => None,
    };

    let cli_config = CliSettings {
        config_path: gosh_configfile,
        workdir: gosh_workdir,
        validate: matches.get_flag("validate"),
        quiet: matches.get_flag("quiet"),
        context,
    };

    Ok(cli_config)
}

pub async fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let cli_settings = settings(matches)?;

    let gosh_config = GoshConfig::from_file(&cli_settings.config_path, &cli_settings.workdir);

    let sbom = Arc::new(Mutex::new(Sbom::default()));
    let git_cache_registry = GitCacheRegistry::default();

    // let grpc_socker_addr = "127.0.0.1:8000".parse().expect("correct address");
    let grpc_socker_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
    let stop_grpc_server =
        grpc_server::run(grpc_socker_addr, sbom.clone(), git_cache_registry).await?;

    tracing::debug!("Dockerfile:\n{}", gosh_config.dockerfile);

    tokio::spawn(async move {
        tracing::info!("Start build...");

        let gosh_builder = GoshBuilder {
            config: gosh_config,
        };

        gosh_builder
            .run(cli_settings.quiet)
            .await
            .expect("image build successful finish");

        tracing::info!("End build...");
    })
    .await
    .unwrap();

    // {
    //     use tokio::io::AsyncBufReadExt;
    //     println!("Press any key...");
    //     tokio::io::stdin().read_u8().await?;
    // }

    tracing::info!("Stoping build server...");
    stop_grpc_server();

    // SBOM

    // TODO: fix SBOM_OUT env var confusion in case of --validate

    if cli_settings.validate {
        tracing::info!("Validate SBOM...");
        let old_bom = cyclonedx_bom::prelude::Bom::parse_from_json_v1_3(
            File::open(sbom::SBOM_DEFAULT_FILE_NAME).expect("SBOM file exists"),
        )
        .expect("Failed to parse BOM");
        let bom = sbom.lock().await.get_bom()?;
        if bom != old_bom {
            tracing::error!("SBOM validation fail");
            anyhow::bail!("SBOM validation fail");
        } else {
            tracing::info!("SBOM validation success");
            return Ok(());
        }
    } else {
        let sbom_path =
            std::env::var("SBOM_OUT").unwrap_or(sbom::SBOM_DEFAULT_FILE_NAME.to_owned());

        tracing::info!("Writing SBOM to {}", sbom_path);
        sbom.lock().await.save_to(sbom_path).await?;
        tracing::info!("SBOM's ready");
    }

    Ok(())
}
