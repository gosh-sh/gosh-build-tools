mod defaults;

use crate::crypto::generate_keypair_from_secret;
use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::{env, fmt};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct UserWalletConfig {
    pub pubkey: String,
    pub secret: String,
    pub profile: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct NetworkConfig {
    #[serde(rename = "user-wallet")]
    user_wallet: Option<UserWalletConfig>,
    #[serde(default = "std::vec::Vec::<String>::new")]
    endpoints: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
    #[serde(rename = "ipfs")]
    ipfs_http_endpoint: String,

    #[serde(rename = "primary-network")]
    primary_network: String,

    #[serde(rename = "networks")]
    networks: HashMap<String, NetworkConfig>,
}

impl fmt::Debug for UserWalletConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserWalletConfig")
            .field("pubkey", &self.pubkey)
            .finish_non_exhaustive()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ipfs_http_endpoint: defaults::IPFS_HTTP_ENDPOINT.to_string(),
            networks: defaults::NETWORK_ENDPOINTS
                .iter()
                .map(|(network, endpoints)| {
                    let network_config = NetworkConfig {
                        user_wallet: None,
                        endpoints: endpoints.to_vec(),
                    };
                    (network.to_owned(), network_config)
                })
                .collect(),
            primary_network: defaults::PRIMARY_NETWORK.to_string(),
        }
    }
}

impl Config {
    fn read<TReader: Read + Sized>(config_reader: TReader) -> anyhow::Result<Self> {
        let config: Config = serde_json::from_reader(config_reader)?;
        Ok(config)
    }

    pub fn load() -> anyhow::Result<Self> {
        let config_path =
            env::var("GOSH_CONFIG_PATH").unwrap_or_else(|_| defaults::CONFIG_LOCATION.to_string());
        let config_path = shellexpand::tilde(&config_path).into_owned();
        let config_path = Path::new(&config_path);
        if !config_path.exists() {
            anyhow::bail!("No gosh config was found");
        }

        let config_file = std::fs::File::open(config_path)?;
        let config_reader = BufReader::new(config_file);
        Self::read(config_reader)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path =
            env::var("GOSH_CONFIG_PATH").unwrap_or_else(|_| defaults::CONFIG_LOCATION.to_string());
        let config_path = shellexpand::tilde(&config_path).into_owned();
        let config_path = Path::new(&config_path);
        let mut file = std::fs::File::create(config_path)?;
        file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    pub fn with_user_data(username: String, secret: String, pubkey: String) -> Self {
        Self {
            ipfs_http_endpoint: defaults::IPFS_HTTP_ENDPOINT.to_string(),
            networks: defaults::NETWORK_ENDPOINTS
                .iter()
                .map(|(network, endpoints)| {
                    let network_config = NetworkConfig {
                        user_wallet: Some(UserWalletConfig {
                            pubkey: pubkey.clone(),
                            secret: secret.clone(),
                            profile: username.clone(),
                        }),
                        endpoints: endpoints.to_vec(),
                    };
                    (network.to_owned(), network_config)
                })
                .collect(),
            primary_network: defaults::PRIMARY_NETWORK.to_string(),
        }
    }

    pub fn check_keys(&self) -> anyhow::Result<()> {
        let network = &self.primary_network;
        let network = self
            .networks
            .get(network)
            .ok_or(anyhow::format_err!("Wrong network configuration"))?
            .clone();
        let profile = network
            .user_wallet
            .ok_or(anyhow::format_err!("No user wallet config"))?;
        let keypair = generate_keypair_from_secret(&profile.secret)?;
        if profile.pubkey != keypair.public {
            anyhow::bail!("Config keypair is invalid");
        }
        Ok(())
    }

    pub fn get_endpoints(&self) -> Vec<String> {
        self.networks.get(&self.primary_network).unwrap().endpoints.clone()
    }

    pub fn get_username(&self) -> Option<UserWalletConfig> {
        self.networks.get(&self.primary_network).unwrap().user_wallet.clone()
    }
}
