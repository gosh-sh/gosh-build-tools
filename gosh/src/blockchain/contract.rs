use ton_client::abi::Abi;

pub struct Contract {
    pub address: BlockchainContractAddress,
    pub abi: Abi,
}

impl GoshContract {
    pub fn new(address: &str, (pretty_name, abi): (&str, &str)) -> Self {
        Contract {
            address: address.into(),
            abi: Abi::Json(abi.to_string()),
        }
    }
}