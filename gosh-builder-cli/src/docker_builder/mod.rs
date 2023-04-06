use gosh_builder_config::GoshConfig;
use std::process::Stdio;
use tokio::{io::AsyncWriteExt, process::Command};

#[async_trait::async_trait]
pub trait ImageBuilder {
    async fn run(&self) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct GoshBuilder {
    pub config: GoshConfig,
    // workdir: PathBuf,
    // docker_file_path: PathBuf,
}

#[async_trait::async_trait]
impl ImageBuilder for GoshBuilder {
    async fn run(&self) -> anyhow::Result<()> {
        let mut command = Command::new("docker");
        command.arg("buildx").arg("build");
        command.arg("--progress=plain");
        command.arg("--no-cache");
        command.arg("--network=host");
        command.arg("--file").arg("-"); // use stdin
        command.arg("--tag").arg("gosh-build"); // TODO
        command
            .arg("--build-arg")
            .arg("http_proxy=http://127.0.0.1:8000");
        command
            .arg("--build-arg")
            .arg("https_proxy=http://127.0.0.1:8000");

        println!("{:?}", command);

        let mut process = command.stdin(Stdio::piped()).spawn()?;

        let Some(ref mut stdin) = process.stdin else {
            anyhow::bail!("Can't take stdin");
        };
        stdin.write_all(self.config.dockerfile.as_bytes()).await?;
        stdin.flush().await?;

        process.wait().await?;
        Ok(())
    }
}
