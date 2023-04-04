mod builder;
mod cli;
mod grpc_server;
mod sbom;

use std::{
    fs::File,
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::{io::AsyncReadExt, sync::Mutex};

use crate::sbom::Sbom;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_settings = cli::settings()?;
    println!("{:?}", cli_settings);

    let build_config = builder::config::parse(&cli_settings.config_path)?;
    println!("{:?}", build_config);

    let sbom = Arc::new(Mutex::new(Sbom::default()));

    // let grpc_socker_addr = "127.0.0.1:8000".parse().expect("correct address");
    let grpc_socker_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
    let stop_grpc_server = grpc_server::run(grpc_socker_addr, sbom.clone()).await?;

    let dockerfile_path =
        builder::config::clean_dockerfile_path(&build_config.docker_file, &cli_settings.workdir)?;
    println!("Dockerfile {:?}", dockerfile_path);

    tokio::spawn(async move {
        println!("Start build...");

        builder::run(&cli_settings.workdir, &dockerfile_path).await;

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
