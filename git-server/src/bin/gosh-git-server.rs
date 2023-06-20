use std::sync::Arc;

use git_registry::registry::GitCacheRegistry;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    gosh_utils::tracing::default_init();

    let git_registry = Arc::new(GitCacheRegistry::default());

    git_server::run(("0.0.0.0", 8080), None, git_registry).await
}
