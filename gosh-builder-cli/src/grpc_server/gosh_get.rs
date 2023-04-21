use crate::{sbom::Sbom, tracing_pipe::MapPerLine, zstd::ZstdReadToEnd};
use gosh_builder_grpc_api::proto::{
    gosh_get_server::GoshGet, CommitRequest, CommitResponse, FileRequest, FileResponse,
};
use std::{path::PathBuf, process::Stdio, sync::Arc};
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
        grpc_request: tonic::Request<CommitRequest>,
    ) -> std::result::Result<tonic::Response<CommitResponse>, tonic::Status> {
        let request = grpc_request.into_inner();

        tracing::debug!("{:?}", request);

        match get_commit(&request.gosh_url, &request.commit).await {
            Ok(body) => {
                // TODO: (maybe?) convert request.commit to canonical hash
                self.sbom
                    .lock()
                    .await
                    .append(format!("{} {}", &request.gosh_url, &request.commit));
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

        match get_file(&request.gosh_url, &request.commit, &request.path).await {
            Ok(body) => {
                // TODO: (maybe?) convert request.commit to canonical hash
                self.sbom.lock().await.append(format!(
                    "{} {} {}",
                    &request.gosh_url, &request.commit, &request.path
                ));
                return Ok(tonic::Response::new(FileResponse { body }));
            }
            Err(error) => return Err(tonic::Status::internal(format!("{:?}", error))),
        }
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

    tracing::debug!("{:?}", &git_context_dir);
    let mut git_clone_process = tokio::process::Command::new("git")
        .arg("clone")
        .arg(gosh_url.as_ref())
        .arg(".") // clone into current dir
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&git_context_dir)
        .spawn()?;

    git_clone_process
        .stdout
        .take()
        .map(|io| io.map_per_line(|line| tracing::debug!("git clone {}", line)));

    git_clone_process
        .stderr
        .take()
        .map(|io| io.map_per_line(|line| tracing::debug!("git clone {}", line)));

    let status = git_clone_process.wait().await?;

    if !status.success() {
        tracing::error!("git-archive process failed: id={}", &id);
        anyhow::bail!("git clone process failed")
    }

    let mut git_archive_process = tokio::process::Command::new("git")
        .arg("archive")
        .arg("--format=tar")
        .arg(commit.as_ref())
        .current_dir(&git_context_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    git_archive_process
        .stderr
        .take()
        .map(|io| io.map_per_line(|line| tracing::debug!("{}", line)));

    let Some(stdout) = git_archive_process.stdout.take() else {
        tracing::error!("unable to take STDOUT: id={}", &id);
        anyhow::bail!("internal error");
    };

    let zstd_body = stdout.zstd_read_to_end().await?;

    if git_archive_process.wait().await?.success() {
        Ok(zstd_body)
    } else {
        tracing::error!(
            "git-archive process failed: id={} zstd_body={}",
            &id,
            zstd_body.len()
        );
        anyhow::bail!("git-archive process failed")
    }
}

async fn get_file(
    gosh_url: impl AsRef<str>,
    commit: impl AsRef<str>,
    file_path: impl AsRef<str>,
) -> anyhow::Result<Vec<u8>> {
    let id = uuid::Uuid::new_v4();
    let git_context_dir: PathBuf = std::env::current_dir()
        .expect("current dir expected")
        .join(".git-cache")
        .join(id.to_string());

    std::fs::create_dir_all(&git_context_dir)
        .expect("create specific directories and their parents");

    let mut git_clone_process = tokio::process::Command::new("git")
        .arg("clone")
        .arg(gosh_url.as_ref())
        .arg(".") // clone into current dir
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&git_context_dir)
        .spawn()?;

    git_clone_process
        .stdout
        .take()
        .map(|io| io.map_per_line(|line| tracing::debug!("{}", line)));

    git_clone_process
        .stderr
        .take()
        .map(|io| io.map_per_line(|line| tracing::debug!("{}", line)));

    git_clone_process.wait().await?;

    let mut git_archive_process = tokio::process::Command::new("git")
        .arg("show")
        .arg(format!("{}:{}", commit.as_ref(), file_path.as_ref()))
        .current_dir(&git_context_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    git_archive_process
        .stderr
        .take()
        .map(|io| io.map_per_line(|line| tracing::debug!("{}", line)));

    let Some(stdout) = git_archive_process.stdout.take() else {
        tracing::error!("unable to take STDOUT: id={}", &id);
        anyhow::bail!("internal error");
    };

    let zstd_body = stdout.zstd_read_to_end().await?;

    if git_archive_process.wait().await?.success() {
        Ok(zstd_body)
    } else {
        anyhow::bail!("git-show process failed (usually it's because file doesn't exist)")
    }
}
