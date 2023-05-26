use crate::abi::{PROFILE, SYSTEM};
use crate::blockchain::call::{call_function, call_getter, is_account_active};
use crate::blockchain::constants::SYSTEM_CONTRACT_ADDESS;
use crate::blockchain::contract::Contract;
use crate::blockchain::ever_client::EverClient;
use colored::Colorize;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

const PROFILE_CHECK_ATTEMPTS: i32 = 10;

pub async fn get_profile_address(
    ever_client: &EverClient,
    username: &str,
) -> anyhow::Result<String> {
    let system_contract = Contract::new(SYSTEM_CONTRACT_ADDESS, SYSTEM);
    let res = call_getter(
        ever_client,
        &system_contract,
        "getProfileAddr",
        Some(json!({ "name": username })),
    )
    .await?;
    Ok(res["value0"]
        .as_str()
        .ok_or(anyhow::format_err!("Failed to decode profile address"))?
        .to_string())
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
        ever_client,
        &profile_contract,
        "isPubkeyCorrect",
        Some(json!({ "pubkey": pubkey })),
    )
    .await?;
    res["value0"]
        .as_bool()
        .ok_or(anyhow::format_err!("Failed to decode getter result"))
}

pub async fn does_profile_exist(ever_client: &EverClient, username: &str) -> anyhow::Result<bool> {
    let address = get_profile_address(ever_client, username).await?;
    is_account_active(ever_client, &address).await
}

pub async fn deploy_profile(
    ever_client: &EverClient,
    username: &str,
    pubkey: &str,
) -> anyhow::Result<()> {
    println!("Start deployment of the profile:");
    println!("  username: {}", username.bright_blue());
    println!("  pubkey: {}", pubkey.bright_blue());
    println!("Please wait...");
    let pubkey = format!("0x{}", pubkey);
    let address = get_profile_address(ever_client, username).await?;
    let system_contract = Contract::new(SYSTEM_CONTRACT_ADDESS, SYSTEM);
    call_function(
        ever_client,
        &system_contract,
        "deployProfile",
        Some(json!({
            "name": username,
            "pubkey": pubkey
        })),
    )
    .await?;

    let mut attempt = 0;
    loop {
        attempt += 1;
        sleep(Duration::from_secs(5)).await;
        match is_account_active(ever_client, &address).await {
            Err(e) => return Err(e),
            Ok(false) => {}
            Ok(true) => {
                break;
            }
        }
        if attempt == PROFILE_CHECK_ATTEMPTS {
            anyhow::bail!("Failed to deploy user profile");
        }
    }
    println!("{}", "\nProfile was successfully deployed\n".green());
    Ok(())
}
