use git_registry::registry::GitCacheRegistry;
use gosh_builder_grpc_api::proto::{
    gosh_get_server::GoshGet, CommitRequest, CommitResponse, FileRequest, FileResponse,
};
use gosh_sbom::{gosh_classification::GoshClassification, Sbom};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct GoshGetService {
    pub sbom: Arc<Mutex<Sbom>>,
    pub git_cache_registry: Arc<GitCacheRegistry>,
}

impl GoshGetService {
    pub fn new(sbom: Arc<Mutex<Sbom>>, git_cache_registry: Arc<GitCacheRegistry>) -> Self {
        Self {
            sbom,
            git_cache_registry,
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

        let commit_hash = self
            .git_cache_registry
            .normalized_commit(&request.gosh_url, &request.commit)
            .await
            .map_err(|error| tonic::Status::internal(format!("{:?}", error)))?;

        let body = self
            .git_cache_registry
            .git_archive(&request.gosh_url, &commit_hash)
            .await
            .map_err(|error| tonic::Status::internal(format!("{:?}", error)))?;

        self.sbom.lock().await.append(
            GoshClassification::Commit,
            format!("{}:{}", &request.gosh_url, &commit_hash),
        );

        return Ok(tonic::Response::new(CommitResponse { body }));
    }

    async fn file(
        &self,
        grpc_request: tonic::Request<FileRequest>,
    ) -> std::result::Result<tonic::Response<FileResponse>, tonic::Status> {
        let request = grpc_request.into_inner();

        let commit_hash = self
            .git_cache_registry
            .normalized_commit(&request.gosh_url, &request.commit)
            .await
            .map_err(|error| tonic::Status::internal(format!("{:?}", error)))?;

        let body = self
            .git_cache_registry
            .git_show(&request.gosh_url, &commit_hash, &request.path)
            .await
            .map_err(|error| tonic::Status::internal(format!("{:?}", error)))?;

        self.sbom.lock().await.append(
            GoshClassification::File,
            format!("{}:{}:{}", &request.gosh_url, &commit_hash, &request.path),
        );

        Ok(tonic::Response::new(FileResponse { body }))
    }
}
