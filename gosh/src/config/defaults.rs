use cached::once_cell::sync::Lazy;
use std::{collections::HashMap, vec::Vec};

pub const PRIMARY_NETWORK: &str = "network.gosh.sh";
pub const IPFS_HTTP_ENDPOINT: &str = "https://ipfs.network.gosh.sh";

#[cfg(target_family = "unix")]
pub const CONFIG_LOCATION: &str = "~/.gosh/config.json";

#[cfg(target_family = "windows")]
pub const CONFIG_LOCATION: &str = "$HOME/.gosh/config.json";

pub static NETWORK_ENDPOINTS: Lazy<HashMap<String, Vec<String>>> = Lazy::new(|| {
    HashMap::from([(
        "network.gosh.sh".to_owned(),
        vec![
            "https://bhs01.network.gosh.sh".to_owned(),
            "https://eri01.network.gosh.sh".to_owned(),
            "https://gra01.network.gosh.sh".to_owned(),
        ],
    )])
});
