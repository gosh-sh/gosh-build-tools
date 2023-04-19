mod cli;
use cli::Commands;
use gosh_builder_grpc_api::proto::{gosh_get_client::GoshGetClient, CommitRequest, FileRequest};

const GRPC_URL: &str = "http://localhost:8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app_cli = cli::init()?;

    tracing::trace!("Start grpc client on: {}", GRPC_URL);
    let mut grpc_client = GoshGetClient::connect(GRPC_URL).await?;

    let request = CommitRequest::default();
    let _res = grpc_client.commit(request).await?;

    match &app_cli.command {
        Commands::Commit { gosh_url, commit } => {
            let res = grpc_client
                .commit(CommitRequest {
                    gosh_url: gosh_url.to_owned(),
                    commit: commit.to_owned(),
                })
                .await?;

            tracing::trace!("decode tarball");
            let zstd_content = res.get_ref().body.as_slice();
            let tar = zstd::Decoder::new(zstd_content)?;
            let mut archive = tar::Archive::new(tar);
            tracing::trace!("unpack tarball");
            let local_git_dir = std::env::current_dir()?;
            archive.unpack(&local_git_dir)?;
        }
        Commands::File {
            gosh_url,
            commit,
            path,
        } => {
            let _res = grpc_client
                .file(FileRequest {
                    gosh_url: gosh_url.to_owned(),
                    commit: commit.to_owned(),
                    path: path.to_owned(),
                })
                .await?;
            todo!()
        }
    }

    Ok(())
}
