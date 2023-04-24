use super::GitCacheRepo;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct GitCacheRegistry {
    inner: Mutex<HashMap<String, Arc<Mutex<GitCacheRepo>>>>,
}

impl GitCacheRegistry {
    pub async fn git_archive(
        &self,
        url: impl AsRef<str>,
        commit: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        self.get_or_create_repository(url)
            .await?
            .lock()
            .await
            .git_archive(commit)
            .await
    }

    pub async fn git_show(
        &self,
        url: impl AsRef<str>,
        commit: impl AsRef<str>,
        file_path: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        self.get_or_create_repository(url)
            .await?
            .lock()
            .await
            .git_show(commit, file_path)
            .await
    }

    async fn get_or_create_repository(
        &self,
        url: impl AsRef<str>,
    ) -> anyhow::Result<Arc<Mutex<GitCacheRepo>>> {
        let mut registry_guard = self.inner.lock().await;

        if let Some(git_repo) = registry_guard.get(url.as_ref()) {
            return Ok(git_repo.clone());
        } else {
            let git_repo = Arc::new(Mutex::new(GitCacheRepo::from(url.as_ref().to_owned())));
            registry_guard.insert(url.as_ref().to_owned(), git_repo.clone());

            let git_repo_guard = git_repo.lock().await;

            // ensure that we have the first git repo lock right after we add the repo to the registry
            // but since git_repo_update can take long time we don't want to block whole registry
            drop(registry_guard);

            git_repo_guard.update().await?;

            return Ok(git_repo.clone());
        }
    }
}
