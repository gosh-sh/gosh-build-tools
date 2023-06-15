mod directory;

use actix_files::NamedFile;
use actix_web::{
    error::ErrorInternalServerError, get, web, App, HttpRequest, HttpServer, Responder,
};
use git_registry::registry::GitCacheRegistry;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    gosh_utils::tracing::default_init();

    let git_registry = web::Data::new(GitCacheRegistry::default());

    HttpServer::new(move || {
        App::new()
            .app_data(git_registry.clone())
            .service(web::scope("/{contract}/{dao}/{repo}").service(dumb))
            // .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/{src:.*}")]
async fn dumb(
    req: HttpRequest,
    git_registry: web::Data<GitCacheRegistry>,
    path: web::Path<(String, String, String, String)>,
) -> actix_web::Result<impl Responder> {
    let (contract, dao, repo, src) = path.into_inner();
    tracing::info!(?contract, ?dao, ?repo, ?src);

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

// #[get("/{contract}/{dao}/{repo}/info/refs")]
// async fn refs(
//     git_registry: web::Data<GitCacheRegistry>,
//     path: web::Path<(String, String, String)>,
// ) -> impl Responder {
//     let (contract, dao, repo) = path.into_inner();
//     tracing::info!(?contract, ?dao, ?repo);

//     let url = format!("gosh://{contract}/{dao}/{repo}");
//     git_registry.update_server_info(url).await.unwrap();

//     "".to_string()
// }

// #[get("/{dao}/{repo}")]
// async fn index(path: web::Path<(String, String)>) -> impl Responder {
//     let (dao, repo) = path.into_inner();
//     tracing::info!(?dao, ?repo);
//     "".to_string()
// }

// #[get("/{dao}/{repo}/objects/{id_head}/{id_tail}")]
// async fn objects(path: web::Path<(String, String, String, String)>) -> impl Responder {
//     let (dao, repo, id_head, id_tail) = path.into_inner();
//     tracing::info!(?dao, ?repo, ?id_head, ?id_tail);
//     "".to_string()
// }
