mod git_remote;

use self::git_remote::{GitRemotePool, GitRemoteProcess};
use gosh_builder_grpc_api::proto::{
    git_remote_gosh_server::{GitRemoteGosh, GitRemoteGoshServer},
    CommandRequest, CommandResponse, GetArchiveRequest, GetArchiveResponse, SpawnRequest,
    SpawnResponse,
};
use std::{cell::RefCell, net::SocketAddr};
use tokio::sync::Mutex;
use tonic::transport::Server;

#[derive(Debug, Default)]
pub struct GoshGrpc {
    pub gosh_remote_pool: Mutex<RefCell<GitRemotePool>>,
}

#[tonic::async_trait]
impl GitRemoteGosh for GoshGrpc {
    async fn spawn(
        &self,
        request: tonic::Request<SpawnRequest>,
    ) -> Result<tonic::Response<SpawnResponse>, tonic::Status> {
        todo!()
    }

    async fn command(
        &self,
        request: tonic::Request<CommandRequest>,
    ) -> Result<tonic::Response<CommandResponse>, tonic::Status> {
        eprintln!("Call received");
        let r = request.into_inner();
        eprintln!("Body {:?}", r.body);
        eprintln!(
            "Body try string {:?}",
            String::from_utf8(r.body.clone()).unwrap()
        );

        let arc_git_remote_process = self
            .gosh_remote_pool
            .lock()
            .await
            .get_mut()
            .get_process(&r.id)
            .clone();

        let git_remote_process = arc_git_remote_process.lock().await;

        match git_remote_process.call("test".into()).await {
            Ok(output) => return Ok(tonic::Response::new(CommandResponse { body: output })),
            Err(error) => return Err(tonic::Status::internal(format!("{:?}", error))),
        }
    }

    async fn get_archive(
        &self,
        request: tonic::Request<GetArchiveRequest>,
    ) -> Result<tonic::Response<GetArchiveResponse>, tonic::Status> {
        todo!()
    }
}

pub async fn run(address: SocketAddr) -> anyhow::Result<Box<dyn FnOnce() -> ()>> {
    let grpc = GoshGrpc::default();

    // for shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    println!("Start gRPC");
    let server = Server::builder()
        .add_service(GitRemoteGoshServer::new(grpc))
        .serve_with_shutdown(address, async move {
            rx.await.ok();
            println!("gRPC reveived shutdown");
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
