use crate::raw_config::{clean_dockerfile_path, Dockerfile, RawGoshConfig};
use std::{collections::HashMap, path::Path};

#[derive(Builder, Debug, Clone)]
pub struct GoshConfig {
    pub dockerfile: String,
    pub tag: Option<String>,
    pub args: HashMap<String, String>,
}

impl GoshConfig {
    pub fn from_file(path: impl AsRef<Path>, workdir: impl AsRef<Path>) -> Self {
        let raw_config = RawGoshConfig::try_from_file(path).expect("read gosh file yaml");

        let mut builder = GoshConfigBuilder::default();
        match raw_config.dockerfile {
            Dockerfile::Content(content) => builder.dockerfile(content),
            Dockerfile::Path { ref path } => {
                let dockerfile_path =
                    clean_dockerfile_path(path, workdir.as_ref()).expect("clean dockerfile path");
                let content =
                    std::fs::read_to_string(dockerfile_path).expect("read Dockerfile successful");
                builder.dockerfile(content)
            }
        };
        builder.tag(raw_config.tag);
        if let Some(ref args) = raw_config.args {
            builder.args(args.clone());
        };
        builder.build().expect("gosh config builder")
    }
}
