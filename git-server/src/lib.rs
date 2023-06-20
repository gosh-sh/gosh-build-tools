mod directory;

use actix_files::NamedFile;
use actix_web::{
    error::ErrorInternalServerError, get, web, App, HttpRequest, HttpServer, Responder,
};
use git_registry::registry::GitCacheRegistry;
use gosh_sbom::{gosh_classification::GoshClassification, Sbom};
use std::{net::ToSocketAddrs, sync::Arc};
use tokio::sync::Mutex;

pub async fn run(
    address: impl ToSocketAddrs,
    sbom: Option<Arc<Mutex<Sbom>>>,
    git_registry: Arc<GitCacheRegistry>,
) -> Result<(), std::io::Error> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(git_registry.clone()))
            .app_data(web::Data::new(sbom.clone()))
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
    sbom: web::Data<Option<Arc<Mutex<Sbom>>>>,
    git_registry: web::Data<Arc<GitCacheRegistry>>,
    path: web::Path<(String, String, String, String)>,
) -> actix_web::Result<impl Responder> {
    let (contract, dao, repo, src) = path.into_inner();
    tracing::debug!(?contract, ?dao, ?repo, ?src);

    let gosh_url = format!("gosh://{contract}/{dao}/{repo}");
    git_registry
        .update_server_info(&gosh_url)
        .await
        .map_err(ErrorInternalServerError)?;

    let path = git_registry
        .dumb(&gosh_url, src)
        .await
        .map_err(ErrorInternalServerError)?;

    if let Some(ref s) = sbom.get_ref() {
        s.lock()
            .await
            .append(GoshClassification::Repository, gosh_url)
    };

    if path.is_dir() {
        tracing::debug!(?path, "serve dir");
        Ok(directory::directory_listing(path, &req)?)
    } else {
        tracing::debug!(?path, "serve file");
        Ok(NamedFile::open_async(path).await?.into_response(&req))
    }
}
