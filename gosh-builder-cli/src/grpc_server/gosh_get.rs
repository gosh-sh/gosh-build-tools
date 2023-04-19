use crate::sbom::Sbom;
use gosh_builder_grpc_api::proto::{
    gosh_get_server::GoshGet, CommitRequest, CommitResponse, FileRequest, FileResponse,
};
use std::{path::PathBuf, process::Stdio, sync::Arc};
use tokio::{io::AsyncReadExt, sync::Mutex};

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
        grpc_request: tonic::Request<CommitRequest>,
    ) -> std::result::Result<tonic::Response<CommitResponse>, tonic::Status> {
        let result = grpc_request.into_inner();

        match get_commit(result.gosh_url, result.commit).await {
            Ok(body) => return Ok(tonic::Response::new(CommitResponse { body })),
            Err(error) => return Err(tonic::Status::internal(format!("{:?}", error))),
        }
    }
    async fn file(
        &self,
        grpc_request: tonic::Request<FileRequest>,
    ) -> std::result::Result<tonic::Response<FileResponse>, tonic::Status> {
        let _result = grpc_request.into_inner();
        todo!()
    }
}

async fn get_commit(gosh_url: impl AsRef<str>, commit: impl AsRef<str>) -> anyhow::Result<Vec<u8>> {
    let id = uuid::Uuid::new_v4();
    let git_context_dir: PathBuf = std::env::current_dir()
        .expect("current dir expected")
        .join(".git-cache")
        .join(id.to_string());

    std::fs::create_dir_all(&git_context_dir)
        .expect("create specific directories and their parents");

    let _ = tokio::process::Command::new("git")
        .arg("clone")
        .arg(gosh_url.as_ref())
        .arg(".") // clone into current dir
        .current_dir(&git_context_dir)
        .status()
        .await
        .expect("git clone");

    let mut process = tokio::process::Command::new("git")
        .arg("archive")
        .arg("--format=tar")
        .arg(commit.as_ref())
        .current_dir(&git_context_dir)
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdout = process.stdout.take().expect("stdout intercepted");

    let mut res = Vec::new();
    let mut res2 = Vec::new();

    stdout.read_to_end(&mut res).await?;

    let cursor = std::io::Cursor::new(res);
    zstd::stream::copy_encode(cursor, &mut res2, 0)?;

    process.wait().await?;

    Ok(res2)
}
