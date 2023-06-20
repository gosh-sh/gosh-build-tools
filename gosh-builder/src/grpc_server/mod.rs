mod git_remote_gosh;
mod gosh_get;

use crate::grpc_server::{git_remote_gosh::GitRemoteGoshService, gosh_get::GoshGetService};
use git_registry::registry::GitCacheRegistry;
use gosh_builder_grpc_api::proto::{
    git_remote_gosh_server::GitRemoteGoshServer, gosh_get_server::GoshGetServer,
};
use gosh_sbom::Sbom;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tonic::transport::Server;

pub async fn run(
    address: SocketAddr,
    sbom: Arc<Mutex<Sbom>>,
    git_cache_registry: Arc<GitCacheRegistry>,
) -> anyhow::Result<Box<dyn FnOnce()>> {
    let git_remote_gosh_service = GitRemoteGoshService::new(sbom.clone());
    let gosh_get_service = GoshGetService::new(sbom, git_cache_registry);

    // for shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tracing::info!("Start gRPC on {}", address);
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
