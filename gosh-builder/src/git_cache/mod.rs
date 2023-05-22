pub mod registry;

use crate::{tracing_pipe::MapPerLine, zstd::ZstdReadToEnd};
use std::{collections::hash_map::DefaultHasher, hash::Hasher, path::PathBuf, process::Stdio};

#[derive(Debug)]
struct GitCacheRepo {
    pub git_dir: PathBuf,
    pub url: String,
}

impl GitCacheRepo {
    fn from(url: String) -> Self {
        let repo_url_hash = hex_hash(&url);
        let git_dir = dirs::cache_dir()
            .unwrap_or(PathBuf::from(".cache"))
            .join("gosh")
            .join(repo_url_hash);
        Self { git_dir, url }
    }

    async fn update(&self) -> anyhow::Result<()> {
        if self.git_dir.exists() {
            // TODO: test that repo is not hijaked
            // try git pull
            tracing::info!("git-cache: repo dir exists, try to pull {}", self.url);
            tracing::debug!("{:?}", &self.git_dir);
            let mut git_pull_process = tokio::process::Command::new("git")
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
                tracing::error!(
                    "git pull process failed: url={} dir={:?}",
                    &self.url,
                    &self.git_dir
                );
                anyhow::bail!("git pull process failed")
            }
        } else {
            // git clone
            std::fs::create_dir_all(&self.git_dir)?;

            tracing::debug!("{:?}", &self.git_dir);
            let mut git_clone_process = tokio::process::Command::new("git")
                .arg("clone")
                .arg(&self.url)
                .arg(".") // clone into current dir
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(&self.git_dir)
                .spawn()?;

            if let Some(io) = git_clone_process.stdout.take() {
                io.map_per_line(|line| tracing::debug!("git clone {}", line))
            }

            if let Some(io) = git_clone_process.stderr.take() {
                io.map_per_line(|line| tracing::debug!("git clone {}", line))
            }

            let status = git_clone_process.wait().await?;

            if !status.success() {
                tracing::error!(
                    "git clone process failed: url={} dir={:?}",
                    &self.url,
                    &self.git_dir
                );
                anyhow::bail!("git clone process failed")
            }
        }
        Ok(())
    }

    async fn git_archive(&self, commit: impl AsRef<str>) -> anyhow::Result<Vec<u8>> {
        let mut git_archive_process = tokio::process::Command::new("git")
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
            tracing::error!("unable to take STDOUT: git_dir={:?}", &self.git_dir);
            anyhow::bail!("internal error");
        };

        let zstd_body = stdout.zstd_read_to_end().await?;

        if git_archive_process.wait().await?.success() {
            Ok(zstd_body)
        } else {
            tracing::error!("git-archive process failed: zstd_body={}", zstd_body.len());
            anyhow::bail!("git-archive process failed")
        }
    }

    async fn git_show(
        &self,
        commit: impl AsRef<str>,
        file_path: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        let mut command = tokio::process::Command::new("git");
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
    // async fn try_normalize_ref(&self, commit: impl AsRef<str>) -> anyhow::Result<String> {
    //     let mut git_show_process = tokio::process::Command::new("git")
    //         .arg("show")
    //         .arg(format!("{}:{}", commit.as_ref(), file_path.as_ref()))
    //         .current_dir(&self.git_dir)
    //         .stdout(Stdio::piped())
    //         .stderr(Stdio::piped())
    //         .spawn()?;

    //     if let Some(io) = git_show_process.stderr.take() {
    //         io.map_per_line(|line| tracing::debug!("{}", line))
    //     }

    //     let Some(stdout) = git_show_process.stdout.take() else {
    //         tracing::error!("unable to take STDOUT: url={}", &self.url);
    //         anyhow::bail!("internal error");
    //     };
    // }
}

fn hex_hash<H>(hashable: &H) -> String
where
    H: std::hash::Hash + ?Sized,
{
    let mut hasher = DefaultHasher::new();
    hashable.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
