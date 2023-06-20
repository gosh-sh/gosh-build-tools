use std::{net::SocketAddr, sync::Arc};

use git_registry::registry::GitCacheRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gosh_utils::tracing::default_init();

    let git_registry = Arc::new(GitCacheRegistry::default());

    git_server::server(SocketAddr::from(([0, 0, 0, 0], 8080)), None, git_registry)
        .await
        .map_err(|err| {
            tracing::error!("server error: {}", err);
            err
        })?;

    Ok(())
}
