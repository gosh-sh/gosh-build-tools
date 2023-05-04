mod cli;
mod docker_builder;
pub mod git_cache;
mod grpc_server;
mod log;
mod sbom;
mod tracing_pipe;
pub mod utils;
pub mod zstd;

use crate::docker_builder::ImageBuilder;
use crate::git_cache::registry::GitCacheRegistry;
use crate::{docker_builder::GoshBuilder, sbom::Sbom};
use gosh_builder_config::GoshConfig;
use std::fs::File;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::init();

    let cli_settings = cli::settings()?;
    tracing::debug!("{:?}", cli_settings);

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
            .run()
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
