use serde_json::json;
use crate::abi::{PROFILE, SYSTEM};
use crate::blockchain::call::{call_getter, is_account_active};
use crate::blockchain::contract::Contract;
use crate::blockchain::ever_client::EverClient;
use crate::blockchain::r#const::SYSTEM_CONTRACT_ADDESS;

pub async fn get_profile_address(
    ever_client: &EverClient,
    username: &str
) -> anyhow::Result<String> {
    let system_contract = Contract::new(SYSTEM_CONTRACT_ADDESS, SYSTEM);
    let res = call_getter(
        &ever_client,
        &system_contract,
        "getProfileAddr",
        Some(json!({"name": username})),
    ).await?;
    Ok(res["value0"].as_str().ok_or(anyhow::format_err!("Failed to decode profile address"))?.to_string())
}

pub async fn check_profile_pubkey(
    ever_client: &EverClient,
    username: &str,
    pubkey: &str,
) -> anyhow::Result<bool> {
    let profile_address = get_profile_address(ever_client, username).await?;
    let profile_contract = Contract::new(&profile_address, PROFILE);
    let pubkey = format!("0x{}", pubkey);
    let res = call_getter(
        &ever_client,
        &profile_contract,
        "isPubkeyCorrect",
        Some(json!({"pubkey": pubkey})),
    ).await?;
    Ok(res["value0"].as_bool().ok_or(anyhow::format_err!("Failed to decode getter result"))?)
}

pub async fn does_profile_exist(
    ever_client: &EverClient,
    username: &str,
) -> anyhow::Result<bool> {
    let address = get_profile_address(ever_client, username).await?;
    is_account_active(ever_client, &address).await
}
