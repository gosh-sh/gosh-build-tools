use crate::config::Config;
use crate::crypto::{gen_seed_phrase, generate_keypair_from_mnemonic};
use dialoguer::Input;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use tokio::process::Command;
use std::time::Duration;
use serde_json::json;
use ton_client::crypto::KeyPair;
use crate::abi::SYSTEM;
use crate::blockchain::call::{call_function, call_getter, is_account_active};
use crate::blockchain::contract::Contract;
use crate::blockchain::ever_client::create_client;
use crate::blockchain::r#const::SYSTEM_CONTRACT_ADDESS;
use crate::profile::{check_profile_pubkey, does_profile_exist};

static GOSH_YAML: &str = "\
---
dockerfile:
  path: Dockerfile
tag: your-image-tag

";

pub static GOSH_YAML_PATH: &str = "./Gosh.yaml";

async fn generate_config() -> anyhow::Result<Config> {
    let default_config = Config::default();
    let default_client = create_client(&default_config)?;

    let (username, keys) =  loop {
        let username: String = Input::new()
            .with_prompt("Please enter your username")
            .interact_text()?;

        let profile_exists = does_profile_exist(&default_client, &username).await?;

        let input: String = Input::new()
            .with_prompt("Do you have a seed phrase for GOSH? (Y/n)")
            .with_initial_text("n")
            .interact_text()?;
        let keys: KeyPair = if input == "Y" {
            let user_seed: String = Input::new()
                .with_prompt("Please enter your seed phrase")
                .interact_text()?;

            let keys = generate_keypair_from_mnemonic(&user_seed)?;
            if profile_exists {
                match check_profile_pubkey(&default_client, &username, &keys.public).await {
                    Err(e) => { return Err(e) },
                    Ok(true) => { keys },
                    Ok(false) => {
                        println!("Username does already exist and your seed phrase is not correct for this profile.");
                        println!("Please create a unique username or find your old seed phrase.");
                        continue;
                    },
                }
            } else {
                keys
            }
        } else {
            if profile_exists {
                println!("Username does already exist, please create a unique username.");
                continue;
            }
            println!("New seed phrase will be generated for you.");
            let seed = gen_seed_phrase()?;
            println!("\nSeed: {seed}\n");
            println!("Warning: write down and save your seed phrase in private location. Remember that if you lose it you will lose access to your profile.\n");
            let input: String = Input::new()
                .with_prompt("Have you read and understand the warning? (Y/n)")
                .interact_text()?;
            if input != *"Y" {
                println!("Sorry, but in this case we can't continue the onboarding.");
                exit(0);
            }
            generate_keypair_from_mnemonic(&seed)?
        };

        break (username, keys);
    };

    let config = Config::with_user_data(username, keys.secret, keys.public);
    config.save()?;
    Ok(config)
}

pub async fn init_command() -> anyhow::Result<()> {
    let gosh_config = match Config::load() {
        Ok(config) => match config.check().await {
            Ok(_) => config,
            Err(e) => {
                println!("Your local GOSH config is invalid: {e}.");
                let choice: String = Input::new()
                    .with_prompt("Do you want to go through the onboarding process locally? (Y/n)")
                    .with_initial_text("Y")
                    .interact_text()?;
                if choice != *"Y" {
                    println!("Hope to see you soon.");
                    exit(0);
                }
                generate_config().await?
            }
        },
        Err(_) => {
            println!(
                "\
There was no GOSH config found on your PC.
You can go through the onboarding process on web:

    1. Go to https://app.gosh.sh/onboarding
    2. Create a new profile of sign in into an existing one
    3. Open Menu/Settings and save \"Git remote config\" into your $HOME/.gosh/config.json

Otherwise you can pass onboarding locally.\n"
            );
            let choice: String = Input::new()
                .with_prompt("Do you want to go through the process locally? (Y/n)")
                .with_initial_text("n")
                .interact_text()?;
            if choice != *"Y" {
                println!("Hope to see you soon.");
                exit(0);
            }
            generate_config().await?
        }
    };

    let user_data = gosh_config.get_user_data();
    println!("Your GOSH config parameters:");
    println!("username: {}", user_data.profile);
    println!("pubkey: {}", user_data.pubkey);

    check_local_git_remotes(&user_data.profile).await?;

    create_gosh_yaml()?;

    // let username = user_data.profile;
    // let pubkey = format!("0x{}", user_data.pubkey);
    // let ever_client = create_client(&gosh_config)?;
    // let system_contract = Contract::new(SYSTEM_CONTRACT_ADDESS, SYSTEM);
    // let res = call_getter(
    //     &ever_client,
    //     &system_contract,
    //     "getProfileAddr",
    //     Some(json!({"name": username})),
    // ).await?;
    // let profile_address = res["value0"].as_str().expect("Failed to decode profile address");
    // if is_account_active(&ever_client, profile_address).await? {
    //     println!("User profile account is active");
    // } else {
    //     println!("User profile account is not active. Trying to deploy a profile.");
    //
    //     call_function(&ever_client, &system_contract, "deployProfile",
    //                   Some(json!({
    //         "name": username,
    //         "pubkey": pubkey
    //     }))).await?;
    //     tokio::time::sleep(Duration::from_secs(30)).await;
    //     if is_account_active(&ever_client, profile_address).await? {
    //         println!("User profile account is active");
    //     } else {
    //         anyhow::bail!("Failed to deploy profile. Try again later.");
    //     }
    // }
    Ok(())
}

async fn check_local_git_remotes(profile: &str) -> anyhow::Result<()> {
    let remotes = Command::new("git").arg("remote").arg("-v").output().await?;
    let error_output = String::from_utf8_lossy(&remotes.stderr).to_string();
    if error_output.contains("not a git repository") {
        println!("Seems like you are not inside a git repository.");
        exit(0);
    }

    let output = String::from_utf8_lossy(&remotes.stdout).to_string();
    if !output.contains("gosh://") {
        println!("Seems like your local repo does not have a remote url directed to GOSH.");
        println!(
            "Please go to https://app.gosh.sh/o/{}/repos to get link to the GOSH repository",
            profile
        );
        println!("and add this link to the list of git remotes:");
        println!("  `git remote add gosh gosh://0:0d5c05d7a63f438b57ede179b7110d3e903f5be3b5f543d3d6743d774698e92c/{}/<repo_name>`", profile);
        exit(0);
    }

    Ok(())
}

fn create_gosh_yaml() -> anyhow::Result<()> {
    let path = Path::new(GOSH_YAML_PATH);
    if !path.exists() {
        let mut file = File::create(GOSH_YAML_PATH)?;
        file.write_all(GOSH_YAML.as_bytes())?;
        println!("Gosh.yaml file was successfully generated.");
    } else {
        println!("You already have the Gosh.yaml file in the current directory.");
    }
    Ok(())
}
