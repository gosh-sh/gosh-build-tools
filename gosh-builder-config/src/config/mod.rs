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
    pub install: Vec<String>,
}

impl GoshConfig {
    pub fn from_file(path: impl AsRef<Path>, workdir: impl AsRef<Path>) -> Self {
        let raw_config = RawGoshConfig::try_from_file(path).expect("read gosh file yaml");

        let mut builder = GoshConfigBuilder::default();

        builder.dockerfile(match raw_config.dockerfile {
            Dockerfile::Content(content) => content,
            Dockerfile::Path { ref path } => {
                let dockerfile_path =
                    clean_dockerfile_path(path, workdir.as_ref()).expect("clean dockerfile path");
                if !dockerfile_path.exists() {
                    tracing::error!("Dockerfile not found: {}", dockerfile_path.display());
                    panic!("Dockerfile not found: {}", dockerfile_path.display());
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

        builder.build().expect("gosh config builder")
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
