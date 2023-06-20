use git_registry::registry::GitCacheRegistry;
use gosh_sbom::Sbom;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

pub fn run(
    address: SocketAddr,
    sbom: Arc<Mutex<Sbom>>,
    git_cache_registry: Arc<GitCacheRegistry>,
) -> anyhow::Result<Box<dyn FnOnce()>> {
    tracing::info!("Start Git Server on {}", address);

    // for shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let server = git_server::server(address, Some(sbom), git_cache_registry)
        .with_graceful_shutdown(async move {
            rx.await.ok();
            tracing::info!("gRPC received shutdown");
        });

    tracing::info!("gRPC ready");

    tokio::spawn(async move {
        server.await.ok();
        tracing::info!("gRPC stopped");
    });

    Ok(Box::new(move || {
        tx.send(()).ok();
    }))
}
