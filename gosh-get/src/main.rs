mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use gosh_builder_grpc_api::proto::{gosh_get_client::GoshGetClient, CommitRequest, FileRequest};
use std::{fs::File, io::Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app_cli = Cli::parse();

    tracing::trace!("Connect to proxy: {}", app_cli.proxy_addr);
    let mut grpc_client = GoshGetClient::connect(app_cli.proxy_addr).await?;

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
            let content = zstd::stream::decode_all(zstd_content)?;
            tracing::trace!("After decode {}", String::from_utf8(content)?);
            target_file.flush()?;
        }
    }

    Ok(())
}
