mod directory;

use actix_files::NamedFile;
use actix_web::{
    error::ErrorInternalServerError, get, web, App, HttpRequest, HttpServer, Responder,
};
use git_registry::registry::GitCacheRegistry;
use std::{net::ToSocketAddrs, sync::Arc};

pub async fn run(
    address: impl ToSocketAddrs,
    git_registry: Arc<GitCacheRegistry>,
) -> Result<(), std::io::Error> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(git_registry.clone()))
            .service(web::scope("/{contract}/{dao}/{repo}").service(dumb))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(address)?
    .run()
    .await
}

#[get("/{src:.*}")]
async fn dumb(
    req: HttpRequest,
    git_registry: web::Data<Arc<GitCacheRegistry>>,
    path: web::Path<(String, String, String, String)>,
) -> actix_web::Result<impl Responder> {
    let (contract, dao, repo, src) = path.into_inner();
    tracing::debug!(?contract, ?dao, ?repo, ?src);

    let url = format!("gosh://{contract}/{dao}/{repo}");
    git_registry
        .update_server_info(&url)
        .await
        .map_err(ErrorInternalServerError)?;

    let path = git_registry
        .dumb(&url, src)
        .await
        .map_err(ErrorInternalServerError)?;

    if path.is_dir() {
        tracing::debug!(?path, "serve dir");
        Ok(directory::directory_listing(path, &req)?)
    } else {
        tracing::debug!(?path, "serve file");
        Ok(NamedFile::open_async(path).await?.into_response(&req))
    }
}
