mod directory;

use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use git_registry::registry::GitCacheRegistry;
use gosh_sbom::{gosh_classification::GoshClassification, Sbom};
use hyper::body::Bytes;
use std::{net::SocketAddr, sync::Arc};
use tokio::{fs::File, io::AsyncReadExt, sync::Mutex};
use tower_http::compression::CompressionLayer;

struct GitServerState {
    pub sbom: Option<Arc<Mutex<Sbom>>>,
    pub git_registry: Arc<GitCacheRegistry>,
}

pub fn server(
    addr: SocketAddr,
    sbom: Option<Arc<Mutex<Sbom>>>,
    git_registry: Arc<GitCacheRegistry>,
) -> hyper::Server<hyper::server::conn::AddrIncoming, axum::routing::IntoMakeService<Router>> {
    let shared_state = Arc::new(GitServerState { sbom, git_registry });
    let router = Router::new()
        .route("/:contract/:dao/:repo/*src", get(handler))
        .with_state(shared_state)
        .layer(CompressionLayer::new());

    axum::Server::bind(&addr).serve(router.into_make_service())
}

async fn handler(
    State(state): State<Arc<GitServerState>>,
    Path((contract, dao, repo, src)): Path<(String, String, String, String)>,
) -> Result<Bytes, StatusCode> {
    let gosh_url = format!("gosh://{contract}/{dao}/{repo}");
    tracing::info!(?contract, ?dao, ?repo, ?src);

    state
        .git_registry
        .update_server_info(&gosh_url)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let path = state
        .git_registry
        .dumb(&gosh_url, src)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!(?path);

    if let Some(ref s) = state.sbom {
        s.lock()
            .await
            .append(GoshClassification::Repository, gosh_url)
    };

    if path.is_dir() {
        tracing::debug!("requested directory: {:?}", path);
        // TODO: handle directory listing but we don't have to
        Err(StatusCode::NOT_FOUND)
    } else {
        tracing::debug!(?path, "serve file");
        let mut buf = Vec::new();
        File::open(path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .read_to_end(&mut buf)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Bytes::from(buf))
    }
}
