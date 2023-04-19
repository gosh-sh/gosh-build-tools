mod cli;
mod docker_builder;
mod grpc_server;
mod log;
mod sbom;
mod tracing_pipe;

use crate::docker_builder::ImageBuilder;
use crate::{docker_builder::GoshBuilder, sbom::Sbom};
use gosh_builder_config::GoshConfig;
use std::{
    fs::File,
    io::Write,
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

    // let grpc_socker_addr = "127.0.0.1:8000".parse().expect("correct address");
    let grpc_socker_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
    let stop_grpc_server = grpc_server::run(grpc_socker_addr, sbom.clone()).await?;

    tracing::debug!("Dockerfile {:?}", gosh_config.dockerfile);

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

    tracing::info!("Writing SBOM...");
    let mut sbom_file = File::create("sbom.log")?;
    for s in sbom.lock().await.inner.iter() {
        writeln!(sbom_file, "{}", s)?;
    }
    sbom_file.flush()?;
    tracing::info!("SBOM's ready");

    Ok(())
}
