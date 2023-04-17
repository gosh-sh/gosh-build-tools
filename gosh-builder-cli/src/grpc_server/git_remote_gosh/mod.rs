mod git_remote_process_pool;

use crate::{grpc_server::git_remote_gosh::git_remote_process_pool::GitRemoteProces, sbom::Sbom};
use git_remote_process_pool::GitRemotePool;
use gosh_builder_grpc_api::proto::{
    git_remote_gosh_server::GitRemoteGosh, CommandRequest, CommandResponse, GetArchiveRequest,
    GetArchiveResponse, SpawnRequest, SpawnResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct GitRemoteGoshService {
    pub gosh_remote_pool: Arc<Mutex<GitRemotePool>>,
    pub sbom: Arc<Mutex<Sbom>>,
}

impl GitRemoteGoshService {
    pub fn new(sbom: Arc<Mutex<Sbom>>) -> Self {
        Self {
            sbom,
            ..Default::default()
        }
    }
}

#[tonic::async_trait]
impl GitRemoteGosh for GitRemoteGoshService {
    async fn spawn(
        &self,
        grpc_request: tonic::Request<SpawnRequest>,
    ) -> Result<tonic::Response<SpawnResponse>, tonic::Status> {
        eprintln!("gRPC: spawn");
        let request = grpc_request.into_inner();

        self.sbom.lock().await.append(request.args.join(" "));

        let process = GitRemoteProces::spawn(&request.id, request.args).await;
        self.gosh_remote_pool
            .lock()
            .await
            .insert(&request.id, process);

        Ok(tonic::Response::new(SpawnResponse::default()))
    }

    async fn command(
        &self,
        grpc_request: tonic::Request<CommandRequest>,
    ) -> Result<tonic::Response<CommandResponse>, tonic::Status> {
        eprintln!("gRPC: command");
        let request = grpc_request.into_inner();
        eprintln!("Body {:?}", request.body);
        eprintln!(
            "Body try string {:?}",
            String::from_utf8(request.body.clone()).unwrap()
        );

        let git_remote_process_arc = {
            let mut _lock = self.gosh_remote_pool.lock().await;
            _lock.get(&request.id)
        };

        let mut git_remote_process = git_remote_process_arc.lock().await;

        match git_remote_process.command(request.body).await {
            Ok(output) => return Ok(tonic::Response::new(CommandResponse { body: output })),
            Err(error) => return Err(tonic::Status::internal(format!("{:?}", error))),
        }
    }

    async fn get_archive(
        &self,
        grpc_request: tonic::Request<GetArchiveRequest>,
    ) -> Result<tonic::Response<GetArchiveResponse>, tonic::Status> {
        eprintln!("gRPC: get archive");
        let request = grpc_request.into_inner();

        let git_remote_process_arc = {
            let mut _lock = self.gosh_remote_pool.lock().await;
            _lock.get(&request.id)
        };

        let git_remote_process = git_remote_process_arc.lock().await;

        match git_remote_process.get_archive().await {
            Ok(output) => return Ok(tonic::Response::new(GetArchiveResponse { body: output })),
            Err(error) => return Err(tonic::Status::internal(format!("{:?}", error))),
        }
    }
}
