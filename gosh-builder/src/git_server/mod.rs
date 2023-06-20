use git_registry::registry::GitCacheRegistry;
use gosh_sbom::Sbom;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

pub fn run(
    address: SocketAddr,
    sbom: Arc<Mutex<Sbom>>,
    git_cache_registry: Arc<GitCacheRegistry>,
) -> anyhow::Result<Box<dyn FnOnce()>> {
    // for shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tracing::info!("Start Git Server on {}", address);
    // let server = Server::builder()
    //     .add_service(GitRemoteGoshServer::new(git_remote_gosh_service))
    //     .add_service(GoshGetServer::new(gosh_get_service))
    //     .serve_with_shutdown(address, async move {
    //         rx.await.ok();
    //         tracing::info!("gRPC received shutdown");
    //     });

    tracing::info!("Git Server ready");

    tokio::spawn(async move {
        git_server::run(address, Some(sbom), git_cache_registry);
        // server.await.ok();
        tracing::info!("Git Server stopped");
    });

    Ok(Box::new(move || {
        tx.send(()).ok();
    }))
}
