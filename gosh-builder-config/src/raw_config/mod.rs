#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::Path};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum Dockerfile {
    Path { path: String },
    Content(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawGoshConfig {
    pub dockerfile: Dockerfile,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, String>>,
}

impl TryFrom<&str> for RawGoshConfig {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(serde_yaml::from_str(value)?)
    }
}

impl RawGoshConfig {
    pub fn try_from_file<P>(config_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let config_path = config_path.as_ref();

        tracing::info!("Read configfile: {:?}", config_path);

        Self::try_from_reader(File::open(config_path)?)
    }

    pub fn try_from_reader(rdr: impl std::io::Read) -> anyhow::Result<Self> {
        serde_yaml::from_reader(rdr).map_err(anyhow::Error::from)
    }
}
