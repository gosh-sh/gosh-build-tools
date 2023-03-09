use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct GoshConfig {
    pub docker_file: String,
}

pub fn parse<P>(config_path: P) -> anyhow::Result<GoshConfig>
where
    P: AsRef<Path>,
{
    println!("Read config from {:?}", config_path.as_ref());
    let config_file = std::fs::File::open(config_path.as_ref())?;
    let config_content: GoshConfig = serde_yaml::from_reader(config_file)?;
    Ok(config_content)
}

pub fn clean_dockerfile_path<D>(raw_dockerfile_path: &str, workdir: D) -> anyhow::Result<PathBuf>
where
    D: AsRef<Path>,
{
    let mut path = PathBuf::from(raw_dockerfile_path);
    if !path.is_absolute() {
        path = workdir.as_ref().join(path);
    }
    Ok(path)
}
