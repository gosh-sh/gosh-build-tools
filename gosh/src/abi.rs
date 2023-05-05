macro_rules! abi {
    ($file: expr) => {
        // NOTE: Check that abi are in resources dir in case of `No such file...` error
        ($file, include_str!(concat!("../resources/", $file)))
    };
}

type Abi = (&'static str, &'static str);

pub static SYSTEM: Abi = abi!("systemcontract.abi.json");
pub static PROFILE: Abi = abi!("profile.abi.json");
