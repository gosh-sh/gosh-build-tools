mod cli;
use std::{fs::File, io::Write};

use cli::Commands;
use gosh_builder_grpc_api::proto::{gosh_get_client::GoshGetClient, CommitRequest, FileRequest};

const GRPC_URL: &str = "http://localhost:8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app_cli = cli::init()?;

    tracing::trace!("Start grpc client on: {}", GRPC_URL);
    let mut grpc_client = GoshGetClient::connect(GRPC_URL).await?;

    match &app_cli.command {
        Commands::Commit { gosh_url, commit } => {
            tracing::info!("Get commit...");
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
            // parse path
            tracing::info!("Parse path...");
            let path_buf = std::path::PathBuf::from(path);
            if !path_buf.is_relative() {
                anyhow::bail!("Path must be relative to the git root");
            }

            let Some(filename) = path_buf.file_name() else {
                anyhow::bail!("Path should end up with filename");
            };

            tracing::info!("Get file...");
            let res = grpc_client
                .file(FileRequest {
                    gosh_url: gosh_url.to_owned(),
                    commit: commit.to_owned(),
                    path: path.to_owned(),
                })
                .await?;

            let mut target_file = std::io::BufWriter::new(File::create(filename)?);
            let zstd_content = res.get_ref().body.as_slice();
            tracing::info!("{:?}", zstd_content);
            // zstd::stream::copy_decode(zstd_content, &mut target_file)?;
            let content = zstd::stream::decode_all(zstd_content)?;
            tracing::info!("After decode {}", String::from_utf8(content)?);
            // let decoder = zstd::Decoder::new(zstd_content)?;
            // let mut decoder_buf = std::io::BufReader::new(decoder);
            // std::io::copy(&mut decoder_buf, &mut target_file)?;
            target_file.flush()?;
        }
    }

    Ok(())
}
