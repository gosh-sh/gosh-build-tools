mod cli;
mod docker_builder;
mod grpc_server;
mod sbom;

use crate::docker_builder::ImageBuilder;
use crate::{docker_builder::GoshBuilder, sbom::Sbom};
use gosh_builder_config::GoshConfig;
use std::{
    fs::File,
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::{io::AsyncReadExt, sync::Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_settings = cli::settings()?;
    println!("{:?}", cli_settings);

    let gosh_config = GoshConfig::from_file(&cli_settings.config_path, &cli_settings.workdir);

    let sbom = Arc::new(Mutex::new(Sbom::default()));

    // let grpc_socker_addr = "127.0.0.1:8000".parse().expect("correct address");
    let grpc_socker_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
    let stop_grpc_server = grpc_server::run(grpc_socker_addr, sbom.clone()).await?;

    println!("Dockerfile {:?}", gosh_config.dockerfile);

    tokio::spawn(async move {
        println!("Start build...");

        let gosh_builder = GoshBuilder {
            config: gosh_config,
        };

        gosh_builder
            .run()
            .await
            .expect("image build successful finish");

        println!("End build...");
    })
    .await
    .unwrap();

    println!("Press any key...");
    tokio::io::stdin().read_u8().await?;

    stop_grpc_server();

    let mut sbom_file = File::create("sbom.log")?;
    for s in sbom.lock().await.inner.iter() {
        writeln!(sbom_file, "{}", s)?;
    }
    sbom_file.flush()?;

    Ok(())
}
