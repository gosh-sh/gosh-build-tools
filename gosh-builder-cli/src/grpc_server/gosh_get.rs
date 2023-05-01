use crate::{
    git_cache::registry::GitCacheRegistry,
    sbom::{gosh_classification::GoshClassification, Sbom},
};
use gosh_builder_grpc_api::proto::{
    gosh_get_server::GoshGet, CommitRequest, CommitResponse, FileRequest, FileResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct GoshGetService {
    pub sbom: Arc<Mutex<Sbom>>,
    pub git_cache_registry: GitCacheRegistry,
}

impl GoshGetService {
    pub fn new(sbom: Arc<Mutex<Sbom>>, git_cache_registry: GitCacheRegistry) -> Self {
        Self {
            sbom,
            git_cache_registry,
            ..Default::default()
        }
    }
}

#[tonic::async_trait]
impl GoshGet for GoshGetService {
    async fn commit(
        &self,
        grpc_request: tonic::Request<CommitRequest>,
    ) -> std::result::Result<tonic::Response<CommitResponse>, tonic::Status> {
        let request = grpc_request.into_inner();

        tracing::debug!("{:?}", request);

        match self
            .git_cache_registry
            .git_archive(&request.gosh_url, &request.commit)
            .await
        {
            Ok(body) => {
                // TODO: (maybe?) convert request.commit to canonical hash

                self.sbom.lock().await.append(
                    GoshClassification::Commit,
                    format!("{}:{}", &request.gosh_url, &request.commit),
                );
                return Ok(tonic::Response::new(CommitResponse { body }));
            }
            Err(error) => return Err(tonic::Status::internal(format!("{:?}", error))),
        }
    }
    async fn file(
        &self,
        grpc_request: tonic::Request<FileRequest>,
    ) -> std::result::Result<tonic::Response<FileResponse>, tonic::Status> {
        let request = grpc_request.into_inner();

        match self
            .git_cache_registry
            .git_show(&request.gosh_url, &request.commit, &request.path)
            .await
        {
            Ok(body) => {
                // TODO: (maybe?) convert request.commit to canonical hash
                self.sbom.lock().await.append(
                    GoshClassification::File,
                    format!(
                        "{}:{}:{}",
                        &request.gosh_url, &request.commit, &request.path
                    ),
                );
                return Ok(tonic::Response::new(FileResponse { body }));
            }
            Err(error) => return Err(tonic::Status::internal(format!("{:?}", error))),
        }
    }
}
