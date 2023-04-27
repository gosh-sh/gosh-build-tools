use serde_json::json;
use crate::abi::SYSTEM;
use crate::blockchain::call::call_getter;
use crate::blockchain::contract::Contract;
use crate::blockchain::ever_client::EverClient;
use crate::blockchain::r#const::SYSTEM_CONTRACT_ADDESS;

pub async fn get_profile_address(
    client: &EverClient,
    username: &str
) -> anyhow::Result<String> {
    let system_contract = Contract::new(SYSTEM_CONTRACT_ADDESS, SYSTEM);
    let res = call_getter(
        &ever_client,
        &system_contract,
        "getProfileAddr",
        Some(json!({"name": username})),
    ).await?;
    Ok(res["value0"].as_str().ok_or("Failed to decode profile address")?.to_string())
}

pub async fn get_profile_pubkey(
    client: &EverClient,
    profile_address: &str
) -> anyhow::Result<String>