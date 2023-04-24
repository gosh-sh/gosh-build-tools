use gosh_builder_config::GoshConfig;
use std::process::Stdio;
use tokio::{io::AsyncWriteExt, process::Command};

use crate::tracing_pipe::MapPerLine;

#[async_trait::async_trait]
pub trait ImageBuilder {
    async fn run(&self) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct GoshBuilder {
    pub config: GoshConfig,
}

#[async_trait::async_trait]
impl ImageBuilder for GoshBuilder {
    async fn run(&self) -> anyhow::Result<()> {
        let mut command = Command::new("docker");
        command.arg("buildx").arg("build");
        command.arg("--progress=plain");
        command.arg("--no-cache");
        command.arg("--network=host"); // TODO: fix network access
        if let Some(ref tag) = self.config.tag {
            command.arg("--tag").arg(tag);
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
        command
            .arg("--build-arg")
            .arg("https_proxy=http://127.0.0.1:8000");

        command.arg("-"); // use stdin
        tracing::debug!("{:?}", command);

        let mut process = command
            .stdin(Stdio::piped())
            // .stdout(Stdio::piped())
            // .stderr(Stdio::piped())
            .spawn()?;

        // TODO: make it optional via envs
        // if stdout/stderr are piped we redirect them to tracing
        process
            .stdout
            .take()
            .map(|io| io.map_per_line(|line| tracing::info!("{}", line)));

        process
            .stderr
            .take()
            .map(|io| io.map_per_line(|line| tracing::info!("{}", line)));

        let Some(ref mut stdin) = process.stdin else {
            anyhow::bail!("Can't take stdin");
        };
        stdin.write_all(self.config.dockerfile.as_bytes()).await?;
        stdin.flush().await?;

        process.wait().await?;
        Ok(())
    }
}
