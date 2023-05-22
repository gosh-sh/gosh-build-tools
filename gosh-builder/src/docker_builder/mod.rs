pub mod git_context;

use gosh_builder_config::GoshConfig;
use std::{
    net::SocketAddr,
    process::{ExitStatus, Stdio},
};
use tokio::{io::AsyncWriteExt, process::Command};

use crate::tracing_pipe::MapPerLine;

#[async_trait::async_trait]
pub trait ImageBuilder {
    async fn run(&self, quiet: bool, proxy_socket: &SocketAddr) -> anyhow::Result<ExitStatus>;
}

#[derive(Debug, Clone)]
pub struct GoshBuilder {
    pub config: GoshConfig,
}

#[async_trait::async_trait]
impl ImageBuilder for GoshBuilder {
    async fn run(&self, quiet: bool, proxy_socket: &SocketAddr) -> anyhow::Result<ExitStatus> {
        let mut command = Command::new("docker");
        command.arg("buildx");
        command.arg("build");
        // command.arg("--progress=plain");
        command.arg("--no-cache");
        // command.arg("--allow").arg("network.host");
        // command.arg("--sbom").arg("true");
        command.arg("--network=host"); // TODO: fix network access
        if let Some(ref tag) = self.config.tag {
            command.arg("--tag").arg(tag);
        }

        if quiet {
            command.arg("--quiet");
        }

        // !WARNING! potential security breach
        // self.config.args should be filtered before used
        // e.g. proxy settings

        for (key, value) in &self.config.args {
            command.arg(key).arg(value);
        }

        let proxy_addr = format!("{}:{}", proxy_socket.ip(), proxy_socket.port());

        command
            .arg("--build-arg")
            .arg(format!("http_proxy=http://{}", proxy_addr));
        command
            .arg("--build-arg")
            .arg(format!("https_proxy=http://{}", proxy_addr));

        command
            .arg("--build-arg")
            .arg(format!("GOSH_HTTP_PROXY=http://{proxy_addr}"));

        command.arg("-"); // use stdin
        tracing::debug!("{:?}", command);

        if quiet {
            command.stdin(Stdio::piped());
            command.stdout(Stdio::inherit());
            command.stderr(Stdio::inherit());
        } else {
            command.stdin(Stdio::piped());
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
        }

        let mut process = command.spawn()?;

        // stdout
        if !quiet {
            // IMPORTANT: allow to modify stdout only if not in quiet mode
            if let Some(io) = process.stdout.take() {
                io.map_per_line(|line| println!("{}", line))
            }
        }

        // stderr
        if let Some(io) = process.stderr.take() {
            io.map_per_line(|line| tracing::info!("{}", line))
        }

        // stdin
        let Some(ref mut stdin) = process.stdin else {
            anyhow::bail!("Can't take stdin");
        };
        stdin.write_all(self.config.dockerfile.as_bytes()).await?;
        stdin.flush().await?;

        Ok(process.wait().await?)
    }
}
