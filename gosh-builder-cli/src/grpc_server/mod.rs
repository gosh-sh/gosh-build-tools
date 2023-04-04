mod git_remote;

use crate::sbom::Sbom;

use self::git_remote::{GitRemotePool, GitRemoteProces};
use gosh_builder_grpc_api::proto::{
    git_remote_gosh_server::{GitRemoteGosh, GitRemoteGoshServer},
    CommandRequest, CommandResponse, GetArchiveRequest, GetArchiveResponse, SpawnRequest,
    SpawnResponse,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tonic::transport::Server;

#[derive(Debug, Default)]
pub struct GoshGrpc {
    pub gosh_remote_pool: Arc<Mutex<GitRemotePool>>,
    pub sbom: Arc<Mutex<Sbom>>,
}

impl GoshGrpc {
    fn new(sbom: Arc<Mutex<Sbom>>) -> Self {
        Self {
            sbom,
            ..Default::default()
        }
    }
}

#[tonic::async_trait]
impl GitRemoteGosh for GoshGrpc {
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

pub async fn run(address: SocketAddr, sbom: Arc<Mutex<Sbom>>) -> anyhow::Result<Box<dyn FnOnce()>> {
    let grpc = GoshGrpc::new(sbom);

    // for shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    println!("Start gRPC");
    let server = Server::builder()
        .add_service(GitRemoteGoshServer::new(grpc))
        .serve_with_shutdown(address, async move {
            rx.await.ok();
            println!("gRPC received shutdown");
        });

    println!("gRPC ready");

    tokio::spawn(async move {
        server.await.ok();
        println!("gRPC stopped");
    });

    Ok(Box::new(move || {
        tx.send(()).ok();
    }))
}
