use crate::sbom::Sbom;
use gosh_builder_grpc_api::proto::{
    gosh_get_server::GoshGet, CommitRequest, CommitResponse, FileRequest, FileResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct GoshGetService {
    pub sbom: Arc<Mutex<Sbom>>,
}

impl GoshGetService {
    pub fn new(sbom: Arc<Mutex<Sbom>>) -> Self {
        Self {
            sbom,
            ..Default::default()
        }
    }
}

#[tonic::async_trait]
impl GoshGet for GoshGetService {
    async fn commit(
        &self,
        request: tonic::Request<CommitRequest>,
    ) -> std::result::Result<tonic::Response<CommitResponse>, tonic::Status> {
        todo!()
    }
    async fn file(
        &self,
        request: tonic::Request<FileRequest>,
    ) -> std::result::Result<tonic::Response<FileResponse>, tonic::Status> {
        todo!()
    }
}
