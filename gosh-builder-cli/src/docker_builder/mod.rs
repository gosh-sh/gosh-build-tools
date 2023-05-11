use gosh_builder_config::GoshConfig;
use std::process::Stdio;
use tokio::{io::AsyncWriteExt, process::Command};

use crate::tracing_pipe::MapPerLine;

#[async_trait::async_trait]
pub trait ImageBuilder {
    async fn run(&self, quiet: bool) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct GoshBuilder {
    pub config: GoshConfig,
}

#[async_trait::async_trait]
impl ImageBuilder for GoshBuilder {
    async fn run(&self, quiet: bool) -> anyhow::Result<()> {
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
        command
            .arg("--build-arg")
            .arg("http_proxy=http://127.0.0.1:8000");
        // .arg("http_proxy=http://host.docker.internal:8000");
        command
            .arg("--build-arg")
            .arg("https_proxy=http://127.0.0.1:8000");
        // .arg("https_proxy=http://host.docker.internal:8000");

        command.arg("-"); // use stdin
        tracing::debug!("{:?}", command);

        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut process = command.spawn()?;

        if let Some(io) = process.stdout.take() {
            io.map_per_line(|line| println!("{}", line))
        }

        if let Some(io) = process.stderr.take() {
            io.map_per_line(|line| tracing::info!("{}", line))
        }

        let Some(ref mut stdin) = process.stdin else {
            anyhow::bail!("Can't take stdin");
        };
        stdin.write_all(self.config.dockerfile.as_bytes()).await?;
        stdin.flush().await?;

        process.wait().await?;

        Ok(())
    }
}
