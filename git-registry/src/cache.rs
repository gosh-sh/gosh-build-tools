use gosh_utils::{tracing_pipe::MapPerLine, zstd::ZstdReadToEnd};
use std::{collections::hash_map::DefaultHasher, hash::Hasher, path::PathBuf, process::Stdio};
use tokio::{io::AsyncReadExt, process::Command};

#[derive(Debug)]
pub(crate) struct GitCacheRepo {
    pub git_dir: PathBuf,
    pub url: String,
}

impl GitCacheRepo {
    pub fn from(url: String) -> Self {
        let repo_url_hash = hex_hash(&url);
        let git_dir = dirs::cache_dir()
            .unwrap_or(PathBuf::from(".cache"))
            .join("gosh")
            .join(repo_url_hash);
        Self { git_dir, url }
    }

    pub async fn update(&self) -> anyhow::Result<()> {
        if self.git_dir.exists() {
            // TODO: test that repo is not hijaked
            // try git pull
            tracing::info!("git-cache: repo dir exists, try to pull {}", self.url);
            tracing::debug!("{:?}", &self.git_dir);
            let mut git_pull_process = Command::new("git")
                .arg("pull")
                .arg("--all")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(&self.git_dir)
                .spawn()?;

            if let Some(io) = git_pull_process.stdout.take() {
                io.map_per_line(|line| tracing::debug!("git pull: {}", line))
            }

            if let Some(io) = git_pull_process.stderr.take() {
                io.map_per_line(|line| tracing::debug!("git pull: {}", line))
            }

            let status = git_pull_process.wait().await?;

            if !status.success() {
                anyhow::bail!(
                    "git pull process failed: url={} dir={:?}",
                    &self.url,
                    &self.git_dir
                );
            }
        } else {
            // git clone
            std::fs::create_dir_all(&self.git_dir)?;

            tracing::debug!("{:?}", &self.git_dir);
            let mut git_clone_process = Command::new("git")
                .arg("clone")
                .arg(&self.url)
                .arg(".") // clone into current dir
                .current_dir(&self.git_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            if let Some(io) = git_clone_process.stdout.take() {
                io.map_per_line(|line| tracing::debug!("git clone {}", line))
            }

            if let Some(io) = git_clone_process.stderr.take() {
                io.map_per_line(|line| tracing::debug!("git clone {}", line))
            }

            let status = git_clone_process.wait().await?;

            if !status.success() {
                anyhow::bail!(
                    "git clone process failed: url={} dir={:?}",
                    &self.url,
                    &self.git_dir
                );
            }
        }
        Ok(())
    }

    pub async fn update_server_info(&self) -> anyhow::Result<()> {
        Command::new("git")
            .arg("update-server-info")
            .current_dir(&self.git_dir)
            .output()
            .await?;
        Ok(())
    }

    pub async fn dumb(&self, src: impl AsRef<str>) -> anyhow::Result<PathBuf> {
        Ok(self.git_dir.join(".git").join(src.as_ref()))
    }

    pub async fn git_archive(&self, commit: impl AsRef<str>) -> anyhow::Result<Vec<u8>> {
        let mut git_archive_process = Command::new("git")
            .arg("archive")
            .arg("--format=tar")
            .arg(commit.as_ref())
            .current_dir(&self.git_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(io) = git_archive_process.stderr.take() {
            io.map_per_line(|line| tracing::debug!("{}", line))
        }

        let Some(stdout) = git_archive_process.stdout.take() else {
            anyhow::bail!("unable to take STDOUT: git_dir={:?}", &self.git_dir);
        };

        let zstd_body = stdout.zstd_read_to_end().await?;

        if git_archive_process.wait().await?.success() {
            Ok(zstd_body)
        } else {
            tracing::error!("git-archive process failed: zstd_body={}", zstd_body.len());
            anyhow::bail!("git-archive process failed")
        }
    }

    pub async fn git_show(
        &self,
        commit: impl AsRef<str>,
        file_path: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        let mut command = Command::new("git");
        command
            .arg("show")
            .arg(format!("{}:{}", commit.as_ref(), file_path.as_ref()))
            .current_dir(&self.git_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::trace!("{:?}", command);
        let mut git_show_process = command.spawn()?;

        if let Some(io) = git_show_process.stderr.take() {
            io.map_per_line(|line| tracing::debug!("{}", line))
        }

        let Some(stdout) = git_show_process.stdout.take() else {
            tracing::error!("unable to take STDOUT: url={}", &self.url);
            anyhow::bail!("internal error");
        };

        let zstd_body = stdout.zstd_read_to_end().await?;
        tracing::trace!("zstd_body: {:?}", &zstd_body);

        if git_show_process.wait().await?.success() {
            Ok(zstd_body)
        } else {
            anyhow::bail!("git-show process failed (usually it's because file doesn't exist)")
        }
    }

    pub async fn git_show_uncompressed(
        &self,
        commit: impl AsRef<str>,
        file_path: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        let mut command = Command::new("git");
        command
            .arg("show")
            .arg(format!("{}:{}", commit.as_ref(), file_path.as_ref()))
            .current_dir(&self.git_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::trace!("{:?}", command);
        let mut git_show_process = command.spawn()?;

        if let Some(io) = git_show_process.stderr.take() {
            io.map_per_line(|line| tracing::debug!("{}", line))
        }

        let Some(ref mut stdout) = git_show_process.stdout.take() else {
            tracing::error!("unable to take STDOUT: url={}", &self.url);
            anyhow::bail!("internal error");
        };

        let mut body = Vec::new();
        stdout.read_to_end(&mut body).await?;
        tracing::trace!("body: {:?}", &body);

        if git_show_process.wait().await?.success() {
            Ok(body)
        } else {
            anyhow::bail!("git-show process failed (usually it's because file doesn't exist)")
        }
    }

    pub async fn normalized_commit(&self, commit: impl AsRef<str>) -> anyhow::Result<String> {
        let mut git_process = tokio::process::Command::new("git")
            .arg("rev-list")
            .arg("--no-walk")
            .arg(commit.as_ref())
            .current_dir(&self.git_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(io) = git_process.stderr.take() {
            io.map_per_line(|line| tracing::debug!("{}", line))
        }

        let Some(mut stdout) = git_process.stdout.take() else {
            tracing::error!("unable to take STDOUT: url={}", &self.url);
            anyhow::bail!("internal error");
        };

        let mut body = Vec::new();
        stdout.read_to_end(&mut body).await?;
        tracing::trace!("body: {:?}", &body);

        if git_process.wait().await?.success() {
            Ok(String::from_utf8(body)?.trim().to_string())
        } else {
            anyhow::bail!("can't normalize `{}` to commit hash", commit.as_ref())
        }
    }
}

fn hex_hash<H>(hashable: &H) -> String
where
    H: std::hash::Hash + ?Sized,
{
    let mut hasher = DefaultHasher::new();
    hashable.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
