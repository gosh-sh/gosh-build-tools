mod git_remote_gosh;
mod gosh_get;

use crate::{
    git_cache::registry::GitCacheRegistry,
    grpc_server::{git_remote_gosh::GitRemoteGoshService, gosh_get::GoshGetService},
    sbom::Sbom,
};
use gosh_builder_grpc_api::proto::{
    git_remote_gosh_server::GitRemoteGoshServer, gosh_get_server::GoshGetServer,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tonic::transport::Server;

pub async fn run(
    address: SocketAddr,
    sbom: Arc<Mutex<Sbom>>,
    git_cache_registry: GitCacheRegistry,
) -> anyhow::Result<Box<dyn FnOnce()>> {
    let git_remote_gosh_service = GitRemoteGoshService::new(sbom.clone());
    let gosh_get_service = GoshGetService::new(sbom.clone(), git_cache_registry);

    // for shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tracing::info!("Start gRPC");
    let server = Server::builder()
        .add_service(GitRemoteGoshServer::new(git_remote_gosh_service))
        .add_service(GoshGetServer::new(gosh_get_service))
        .serve_with_shutdown(address, async move {
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
