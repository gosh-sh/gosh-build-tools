use ton_client::abi::Abi;
use ton_client::crypto::KeyPair;

pub struct Contract {
    pub address: String,
    pub abi: Abi,
    keypair: Option<KeyPair>,
}

impl Contract {
    pub fn new(address: &str, (_, abi): (&str, &str)) -> Self {
        Contract {
            address: address.into(),
            abi: Abi::Json(abi.to_string()),
            keypair: None,
        }
    }

    // pub fn set_keys(&mut self, keypair: KeyPair) {
    //     self.keypair = Some(keypair);
    // }

    pub fn get_keys(&self) -> Option<KeyPair> {
        self.keypair.clone()
    }
}
