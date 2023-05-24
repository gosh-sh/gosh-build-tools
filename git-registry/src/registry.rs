use crate::cache::GitCacheRepo;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct GitCacheRegistry {
    inner: Mutex<HashMap<String, Arc<Mutex<GitCacheRepo>>>>,
}

// TODO: make archivation optional

impl GitCacheRegistry {
    pub async fn git_archive(
        &self,
        url: impl AsRef<str>,
        commit: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        tracing::debug!(
            "git_archive: url={:?}, commit={:?}",
            url.as_ref(),
            commit.as_ref()
        );
        let repo = self.get_or_create_repository(url).await?;

        let repo_lock = repo.lock().await;
        // let commit = repo_lock.try_normalize_ref(commit).await?;
        repo_lock.git_archive(commit).await
    }

    pub async fn git_show(
        &self,
        url: impl AsRef<str>,
        commit: impl AsRef<str>,
        file_path: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        tracing::debug!(
            "git_show: url={:?} commit={:?} file_path={:?}",
            url.as_ref(),
            commit.as_ref(),
            file_path.as_ref()
        );
        self.get_or_create_repository(url)
            .await?
            .lock()
            .await
            .git_show(commit, file_path)
            .await
    }

    pub async fn git_show_uncompressed(
        &self,
        url: impl AsRef<str>,
        commit: impl AsRef<str>,
        file_path: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        tracing::debug!(
            "git_show_uncompressed: url={:?} commit={:?} file_path={:?}",
            url.as_ref(),
            commit.as_ref(),
            file_path.as_ref()
        );
        self.get_or_create_repository(url)
            .await?
            .lock()
            .await
            .git_show_uncompressed(commit, file_path)
            .await
    }

    async fn get_or_create_repository(
        &self,
        url: impl AsRef<str>,
    ) -> anyhow::Result<Arc<Mutex<GitCacheRepo>>> {
        let mut registry_guard = self.inner.lock().await;

        if let Some(git_repo) = registry_guard.get(url.as_ref()) {
            Ok(git_repo.clone())
        } else {
            let git_repo = Arc::new(Mutex::new(GitCacheRepo::from(url.as_ref().to_owned())));
            registry_guard.insert(url.as_ref().to_owned(), git_repo.clone());

            let git_repo_guard = git_repo.lock().await;

            // ensure that we have the first git repo lock right after we add the repo to the registry
            // but since git_repo_update can take long time we don't want to block whole registry
            drop(registry_guard);

            git_repo_guard.update().await?;

            Ok(git_repo.clone())
        }
    }
}
