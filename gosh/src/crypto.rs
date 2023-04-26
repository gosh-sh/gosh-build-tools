use crate::blockchain::ever_client::create_client_local;
use ton_client::crypto::{
    default_hdkey_derivation_path, default_mnemonic_word_count, hdkey_derive_from_xprv_path,
    hdkey_secret_from_xprv, hdkey_xprv_from_mnemonic, mnemonic_from_random,
    nacl_sign_keypair_from_secret_key, KeyPair, MnemonicDictionary,
    ParamsOfHDKeyDeriveFromXPrvPath, ParamsOfHDKeySecretFromXPrv, ParamsOfHDKeyXPrvFromMnemonic,
    ParamsOfMnemonicFromRandom, ParamsOfNaclSignKeyPairFromSecret,
};

pub fn gen_seed_phrase() -> anyhow::Result<String> {
    let client = create_client_local()?;
    let phrase = mnemonic_from_random(
        client,
        ParamsOfMnemonicFromRandom {
            dictionary: Some(MnemonicDictionary::English),
            word_count: Some(default_mnemonic_word_count()),
        },
    )
    .map(|r| r.phrase)?;
    Ok(phrase)
}

pub fn generate_keypair_from_mnemonic(mnemonic: &str) -> anyhow::Result<KeyPair> {
    let client = create_client_local()?;
    let hdk_master = hdkey_xprv_from_mnemonic(
        client.clone(),
        ParamsOfHDKeyXPrvFromMnemonic {
            dictionary: Some(MnemonicDictionary::English),
            word_count: Some(default_mnemonic_word_count()),
            phrase: mnemonic.to_string(),
        },
    )?;

    let hdk_root = hdkey_derive_from_xprv_path(
        client.clone(),
        ParamsOfHDKeyDeriveFromXPrvPath {
            xprv: hdk_master.xprv,
            path: default_hdkey_derivation_path(),
        },
    )?;

    let secret = hdkey_secret_from_xprv(
        client.clone(),
        ParamsOfHDKeySecretFromXPrv {
            xprv: hdk_root.xprv,
        },
    )?;

    let mut keypair: KeyPair = nacl_sign_keypair_from_secret_key(
        client,
        ParamsOfNaclSignKeyPairFromSecret {
            secret: secret.secret,
        },
    )?;

    // special case if secret contains public key too.
    let secret = hex::decode(&keypair.secret)?;
    if secret.len() > 32 {
        keypair.secret = hex::encode(&secret[..32]);
    }
    Ok(keypair)
}

pub fn generate_keypair_from_secret(secret: &str) -> anyhow::Result<KeyPair> {
    let client = create_client_local()?;
    let mut keypair: KeyPair = nacl_sign_keypair_from_secret_key(
        client,
        ParamsOfNaclSignKeyPairFromSecret {
            secret: secret.to_string(),
        },
    )?;
    // special case if secret contains public key too.
    let secret = hex::decode(&keypair.secret)?;
    if secret.len() > 32 {
        keypair.secret = hex::encode(&secret[..32]);
    }
    Ok(keypair)
}
