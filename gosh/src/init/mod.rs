use crate::config::Config;
use crate::crypto::{gen_seed_phrase, generate_keypair_from_mnemonic};
use dialoguer::Input;
use std::process::exit;
use std::time::Duration;
use serde_json::json;
use crate::abi::SYSTEM;
use crate::blockchain::call::{call_function, call_getter, is_account_active};
use crate::blockchain::contract::Contract;
use crate::blockchain::ever_client::create_client;
use crate::blockchain::r#const::SYSTEM_CONTRACT_ADDESS;

fn generate_config() -> anyhow::Result<Config> {
    let username: String = Input::new()
        .with_prompt("Please enter your username")
        .interact_text()?;
    let input: String = Input::new()
        .with_prompt("Do you have a seed phrase for GOSH? (Y/n)")
        .with_initial_text("n")
        .interact_text()?;
    let seed: String = if input == "Y" {
        Input::new()
            .with_prompt("Please enter your seed phrase")
            .interact_text()?
    } else {
        println!("New seed will be generated for you.");
        let seed = gen_seed_phrase()?;
        println!("\nSeed: {seed}\n");
        println!("Warning: write down and save your seed phrase in private location. Remember that if you lose it you will lose access to your profile.\n");
        let input: String = Input::new()
            .with_prompt("Have you read and understand the warning? (Y/n)")
            .with_initial_text("Y")
            .interact_text()?;
        if input != "Y".to_string() {
            exit(0);
        }
        seed
    };
    let keys = generate_keypair_from_mnemonic(&seed)?;
    let config = Config::with_user_data(username, keys.secret, keys.public);
    config.save()?;
    Ok(config)
}

pub async fn init_command() -> anyhow::Result<()> {
    let gosh_config = match Config::load() {
        Ok(config) => match config.check_keys() {
            Ok(_) => config,
            Err(_) => {
                println!("Your local GOSH config is invalid.");
                let choice: String = Input::new()
                    .with_prompt("Do you want to go through the onboarding process locally? (Y/n)")
                    .with_initial_text("Y")
                    .interact_text()?;
                if choice != "Y".to_string() {
                    println!("Hope to see you soon.");
                    exit(0);
                }
                generate_config()?
            }
        },
        Err(_) => {
            println!("There was no GOSH config found on your PC.");
            println!("You can go through the onboarding process on web https://app.gosh.sh/onboarding (if you have already done it, you should open Settings and save `Git remote config` on your PC).");
            println!("Otherwise you can pass onboarding locally.\n");
            let choice: String = Input::new()
                .with_prompt("Do you want to go through the process locally? (Y/n)")
                .with_initial_text("Y")
                .interact_text()?;
            if choice != "Y".to_string() {
                println!("Hope to see you soon.");
                exit(0);
            }
            generate_config()?
        }
    };
    let userdata = gosh_config.get_username().expect("Your config doesn't contain user data");
    let username = userdata.profile;
    let pubkey = format!("0x{}", userdata.pubkey);
    let ever_client = create_client(&gosh_config)?;
    let system_contract = Contract::new(SYSTEM_CONTRACT_ADDESS, SYSTEM);
    let res = call_getter(
        &ever_client,
        &system_contract,
        "getProfileAddr",
        Some(json!({"name": username})),
    ).await?;
    let profile_address = res["value0"].as_str().expect("Failed to decode profile address");
    if is_account_active(&ever_client, profile_address).await? {
        println!("User profile account is active");
    } else {
        println!("User profile account is not active. Trying to deploy a profile.");

        call_function(&ever_client, &system_contract, "deployProfile",
                      Some(json!({
            "name": username,
            "pubkey": pubkey
        }))).await?;
        tokio::time::sleep(Duration::from_secs(30)).await;
        if is_account_active(&ever_client, profile_address).await? {
            println!("User profile account is active");
        } else {
            anyhow::bail!("Failed to deploy profile. Try again later.");
        }
    }
    Ok(())
}
