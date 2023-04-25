use std::sync::Arc;
use ton_client::{ClientConfig, ClientContext};

pub type EverClient = Arc<ClientContext>;

pub fn create_client_local() -> anyhow::Result<EverClient> {
    let cli = ClientContext::new(ClientConfig::default())?;
    Ok(Arc::new(cli))
}
