use std::env;
use std::ffi::OsStr;
use std::sync::Arc;
use std::time::Duration;
use ton_client::{ClientConfig, ClientContext};
use ton_client::net::NetworkQueriesProtocol;
use crate::config::Config;
use crate::env::parse_env_or;

// default timeout for all types of operation (e.g. message_processing, wait_for, query)
static DEFAULT_BLOCKCHAIN_TIMEOUT: Duration = Duration::from_secs(15 * 60);
static BLOCKCHAIN_TIMEOUT: &'static str = "GOSH_BLOCKCHAIN_TIMEOUT_SEC";
static MESSAGE_PROCESSING_TIMEOUT: &'static str = "GOSH_MESSAGE_PROCESSING_TIMEOUT_SEC";
static WAIT_FOR_TIMEOUT: &'static str = "GOSH_WAIT_FOR_TIMEOUT_SEC";
static QUERY_TIMEOUT: &'static str = "GOSH_QUERY_TIMEOUT_SEC";

pub type EverClient = Arc<ClientContext>;

pub fn create_client_local() -> anyhow::Result<EverClient> {
    let cli = ClientContext::new(ClientConfig::default())?;
    Ok(Arc::new(cli))
}

pub fn create_client(config: &Config) -> anyhow::Result<EverClient> {
    let endpoints = config
        .find_network_endpoints(network)
        .expect("Unknown network");
    let proto = env::var("GOSH_PROTO")
        .unwrap_or_else(|_| ".git".to_string())
        .to_lowercase();

    let blockchain_timeout = parse_env_or(BLOCKCHAIN_TIMEOUT, DEFAULT_BLOCKCHAIN_TIMEOUT)?;
    let message_processing_timeout = parse_env_or(MESSAGE_PROCESSING_TIMEOUT, blockchain_timeout)?;
    let wait_for_timeout = parse_env_or(WAIT_FOR_TIMEOUT, blockchain_timeout)?;
    let query_timeout = parse_env_or(QUERY_TIMEOUT, blockchain_timeout)?;

    let config = ClientConfig {
        network: ton_client::net::NetworkConfig {
            sending_endpoint_count: endpoints.len() as u8,
            endpoints: if endpoints.is_empty() {
                None
            } else {
                Some(endpoints)
            },
            queries_protocol: if proto.starts_with("http") {
                NetworkQueriesProtocol::HTTP
            } else {
                NetworkQueriesProtocol::WS
            },
            network_retries_count: 5,
            message_retries_count: 10,
            message_processing_timeout: message_processing_timeout.as_millis().try_into()?,
            wait_for_timeout: wait_for_timeout.as_millis().try_into()?,
            query_timeout: query_timeout.as_millis().try_into()?,
            ..Default::default()
        },
        ..Default::default()
    };
    let es_client = ClientContext::new(config)
        .map_err(|e| anyhow::anyhow!("failed to create EverSDK client: {}", e))?;

    Ok(Arc::new(es_client))
}