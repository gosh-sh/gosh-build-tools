use git_registry::{git_context::GitContext, registry::GitCacheRegistry};

use crate::raw_config::{Dockerfile, RawGoshConfig};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Builder, Debug, Clone)]
pub struct GoshConfig {
    pub dockerfile: String,
    pub tag: Option<String>,
    #[builder(default)]
    pub args: HashMap<String, String>,
    #[builder(default)]
    pub install: Vec<String>,
}

impl GoshConfig {
    pub async fn from_git_context(
        git_context: &GitContext,
        config_path: &PathBuf,
        git_cache_registry: &GitCacheRegistry,
    ) -> anyhow::Result<Self> {
        // TODO: fix pessimistic cases
        // 1. abs paths (config shouldn't be absolute)
        // 2. config path can lead out of the git repo dir like '../../../../' many times

        let file_path = PathBuf::from(git_context.sub_dir.as_str()).join(config_path);
        tracing::debug!("Config file_path: {:?}", file_path);

        let mut workdir = file_path.clone();
        workdir.pop();
        tracing::debug!("Config workdir: {:?}", workdir);

        let raw_config = RawGoshConfig::try_from_reader(
            git_cache_registry
                .git_show_uncompressed(
                    git_context.remote.as_str(),
                    git_context.git_ref.as_str(),
                    file_path.to_string_lossy(),
                )
                .await?
                .as_slice(),
        )?;

        let mut builder = GoshConfigBuilder::default();

        builder.dockerfile(match raw_config.dockerfile {
            Dockerfile::Content(content) => content,
            Dockerfile::Path { ref path } => {
                let dockerfile_path = workdir.join(path);
                tracing::debug!("Dockerfile path: {:?}", dockerfile_path);
                String::from_utf8(
                    git_cache_registry
                        .git_show_uncompressed(
                            git_context.remote.as_str(),
                            git_context.git_ref.as_str(),
                            dockerfile_path.to_string_lossy(),
                        )
                        .await?,
                )?
            }
        });
        builder.tag(raw_config.tag);

        if let Some(ref args) = raw_config.args {
            builder.args(args.clone());
        };

        if let Some(ref install) = raw_config.install {
            builder.install(install.clone());
        };

        Ok(builder.build().expect("gosh config builder"))
    }

    pub fn from_file(path: impl AsRef<Path>, workdir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let raw_config = RawGoshConfig::try_from_file(path).expect("read gosh file yaml");

        let mut builder = GoshConfigBuilder::default();

        builder.dockerfile(match raw_config.dockerfile {
            Dockerfile::Content(content) => content,
            Dockerfile::Path { ref path } => {
                let dockerfile_path =
                    clean_dockerfile_path(path, workdir.as_ref()).expect("clean dockerfile path");
                if !dockerfile_path.exists() {
                    anyhow::bail!("Dockerfile not found: {}", dockerfile_path.display());
                }
                std::fs::read_to_string(dockerfile_path).expect("read Dockerfile")
            }
        });
        builder.tag(raw_config.tag);

        if let Some(ref args) = raw_config.args {
            builder.args(args.clone());
        };

        if let Some(ref install) = raw_config.install {
            builder.install(install.clone());
        };

        Ok(builder.build()?)
    }
}

fn clean_dockerfile_path<D>(raw_dockerfile_path: &str, workdir: D) -> anyhow::Result<PathBuf>
where
    D: AsRef<Path>,
{
    let mut path = PathBuf::from(raw_dockerfile_path);
    if !path.is_absolute() {
        path = workdir.as_ref().join(path);
    }
    Ok(path)
}
