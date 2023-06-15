use crate::blockchain::constants::SYSTEM_CONTRACT_ADDESS;
use crate::blockchain::ever_client::create_client;
use crate::config::Config;
use crate::crypto::{gen_seed_phrase, generate_keypair_from_mnemonic};
use colored::Colorize;
use dialoguer::Input;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use tokio::process::Command;
use ton_client::crypto::KeyPair;

use crate::profile::{check_profile_pubkey, deploy_profile, does_profile_exist};

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

    let (username, keys) = loop {
        let username: String = Input::new()
            .with_prompt("Please enter your username")
            .interact_text()?;

        let profile_exists = does_profile_exist(&default_client, &username).await?;

        let input: String = Input::new()
            .with_prompt("Do you have a seed phrase for GOSH? (y/N)")
            .with_initial_text("n")
            .interact_text()?;
        let keys: KeyPair = if input.to_lowercase() == "y" {
            let user_seed: String = Input::new()
                .with_prompt("Please enter your seed phrase")
                .interact_text()?;

            let keys = generate_keypair_from_mnemonic(&user_seed)?;
            if profile_exists {
                println!("Profile with this username already exists, checking that your seed phrase is correct...");
                match check_profile_pubkey(&default_client, &username, &keys.public).await {
                    Err(e) => return Err(e),
                    Ok(true) => {
                        println!("{}", "Phrase is valid".bright_green());
                        keys
                    }
                    Ok(false) => {
                        println!("{}", "\nUsername already exists and your seed phrase is not correct for this profile.\n".red());
                        println!(
                            "{}",
                            "Please create a unique username or find your old seed phrase.\n"
                                .bright_yellow()
                        );
                        continue;
                    }
                }
            } else {
                keys
            }
        } else {
            if profile_exists {
                println!(
                    "{}",
                    "\nUsername already exists, please create a unique username.\n".red()
                );
                continue;
            }
            println!("New seed phrase will be generated for you.");
            let seed = gen_seed_phrase()?;
            println!("\nSeed: {}\n", seed.bright_cyan());
            println!("{}", "Warning: write down and save your seed phrase in a safe location. Remember that if you lose it you will also lose access to your profile.\n".bright_red());
            let input: String = Input::new()
                .with_prompt("Have you read and understand the warning? (y/n)")
                .interact_text()?;
            if input.to_lowercase() != *"y" {
                println!("Sorry, but in this case we can't continue the onboarding.");
                exit(0);
            }
            generate_keypair_from_mnemonic(&seed)?
        };
        break (username, keys);
    };

    let config = Config::with_user_data(username, keys.secret, keys.public);
    config.save()?;

    let userdata = config.get_user_data();
    if !does_profile_exist(&default_client, &userdata.profile).await? {
        deploy_profile(&default_client, &userdata.profile, &userdata.pubkey).await?;
    }
    Ok(config)
}

pub async fn init_command() -> anyhow::Result<()> {
    let gosh_config = match Config::load() {
        Ok(config) => match config.check().await {
            Ok(_) => {
                let user_data = config.get_user_data();
                println!("Your GOSH config parameters:");
                println!("  username: {}", user_data.profile.bright_blue());
                println!("  pubkey: {}", user_data.pubkey.bright_blue());
                config
            }
            Err(e) => {
                println!("Your local GOSH config is invalid: {e}.");
                println!("{}", "\nWarning: if you complete the registration process locally, it will delete the current config file\n".bright_red());
                let choice: String = Input::new()
                    .with_prompt("Do you want to go through the onboarding process locally? (y/n)")
                    .interact_text()?;
                if choice.to_lowercase() != *"y" {
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

    1. Go to {}
    2. Create a new profile or sign in to an existing one
    3. Open Menu/Settings and save \"Git remote config\" into your $HOME/.gosh/config.json

Otherwise you can onboard locally.\n",
                "https://app.gosh.sh/onboarding".bright_green().underline()
            );
            let choice: String = Input::new()
                .with_prompt("Do you want to go through the process locally? (y/N)")
                .with_initial_text("n")
                .interact_text()?;
            if choice.to_lowercase() != *"y" {
                println!("Hope to see you soon.");
                exit(0);
            }
            generate_config().await?
        }
    };

    let user_data = gosh_config.get_user_data();
    check_local_git_remotes(&user_data.profile).await?;

    create_gosh_yaml()?;

    Ok(())
}

async fn check_local_git_remotes(profile: &str) -> anyhow::Result<()> {
    let remotes = Command::new("git").arg("remote").arg("-v").output().await?;
    let error_output = String::from_utf8_lossy(&remotes.stderr).to_string();
    if error_output.contains("not a git repository") {
        println!(
            "{}",
            "\nSeems like you are not inside a git repository.\n".red()
        );
        exit(0);
    }

    let output = String::from_utf8_lossy(&remotes.stdout).to_string();
    if !output.contains("gosh://") {
        println!("\nSeems like your local repo does not have a remote url directed to GOSH.");
        let link = format!("https://app.gosh.sh/o/{}/repos", profile)
            .bright_green()
            .underline();
        println!("Please go to {} to get link to the GOSH repository", link);
        println!("and add this link to the list of git remotes:");
        println!(
            "{}",
            format!(
                "  `git remote add gosh gosh://{}/{}/<repo_name>`",
                SYSTEM_CONTRACT_ADDESS, profile
            )
            .bright_yellow()
        );
        exit(0);
    }

    Ok(())
}

fn create_gosh_yaml() -> anyhow::Result<()> {
    let path = Path::new(GOSH_YAML_PATH);
    if !path.exists() {
        let mut file = File::create(GOSH_YAML_PATH)?;
        file.write_all(GOSH_YAML.as_bytes())?;
        println!(
            "{}",
            "\nGosh.yaml file was successfully generated.\n".bright_green()
        );
    } else {
        println!(
            "{}",
            "\nYou already have the Gosh.yaml file in the current directory.".bright_yellow()
        );
    }
    Ok(())
}
