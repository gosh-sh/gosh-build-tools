use crate::config::Config;
use crate::crypto::{gen_seed_phrase, generate_keypair_from_mnemonic};
use dialoguer::Input;
use std::process::exit;

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
        if input != *"Y" {
            exit(0);
        }
        seed
    };
    let keys = generate_keypair_from_mnemonic(&seed)?;
    let config = Config::with_user_data(username, keys.secret, keys.public);
    config.save()?;
    Ok(config)
}

pub fn init_command() -> anyhow::Result<()> {
    let gosh_config = match Config::load() {
        Ok(config) => match config.check_keys() {
            Ok(_) => config,
            Err(_) => {
                println!("Your local GOSH config is invalid.");
                let choice: String = Input::new()
                    .with_prompt("Do you want to go through the onboarding process locally? (Y/n)")
                    .with_initial_text("Y")
                    .interact_text()?;
                if choice != *"Y" {
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
            if choice != *"Y" {
                println!("Hope to see you soon.");
                exit(0);
            }
            generate_config()?
        }
    };
    println!("{}", serde_json::to_string_pretty(&gosh_config)?);
    Ok(())
}
