use std::sync::Arc;
use ton_client::abi::{encode_message, CallSet, ParamsOfEncodeMessage, Signer};

use crate::blockchain::contract::Contract;
use crate::blockchain::ever_client::EverClient;
use ton_client::net::{query_collection, ParamsOfQueryCollection};
use ton_client::processing::{ParamsOfProcessMessage, ProcessingEvent};
use ton_client::tvm::{run_tvm, ParamsOfRunTvm};

pub async fn call_getter(
    client: &EverClient,
    contract: &Contract,
    function_name: &str,
    args: Option<serde_json::Value>,
) -> anyhow::Result<serde_json::Value> {
    let filter = Some(serde_json::json!({
        "id": { "eq": contract.address }
    }));
    let query = query_collection(
        Arc::clone(client),
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter,
            result: "boc".to_owned(),
            limit: Some(1),
            order: None,
        },
    )
    .await
    .map(|r| r.result)?;

    if query.is_empty() {
        anyhow::bail!(
            "account with address {} not found. Was trying to call {}",
            contract.address,
            function_name,
        );
    }
    let boc: String = serde_json::from_value(query[0]["boc"].clone())?;

    let call_set = match args {
        Some(value) => CallSet::some_with_function_and_input(function_name, value),
        None => CallSet::some_with_function(function_name),
    };

    let encoded = encode_message(
        Arc::clone(client),
        ParamsOfEncodeMessage {
            abi: contract.abi.clone(),
            address: Some(contract.address.clone()),
            call_set,
            signer: Signer::None,
            deploy_set: None,
            processing_try_index: None,
            signature_id: None,
        },
    )
    .await?;

    let result = run_tvm(
        Arc::clone(client),
        ParamsOfRunTvm {
            message: encoded.message,
            account: boc,
            abi: Some(contract.abi.clone()),
            boc_cache: None,
            execution_options: None,
            return_updated_account: None,
        },
    )
    .await
    .map(|r| r.decoded.unwrap())
    .map(|r| r.output.unwrap())?;

    Ok(result)
}

pub async fn is_account_active(client: &EverClient, address: &str) -> anyhow::Result<bool> {
    let filter = Some(serde_json::json!({
        "id": { "eq": address }
    }));
    let query = query_collection(
        Arc::clone(client),
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter,
            result: "acc_type".to_owned(),
            limit: Some(1),
            order: None,
        },
    )
    .await
    .map(|r| r.result)?;
    if query.is_empty() {
        return Ok(false);
    }
    Ok(query[0]["acc_type"].as_i64() == Some(1))
}

pub async fn call_function(
    client: &EverClient,
    contract: &Contract,
    function_name: &str,
    args: Option<serde_json::Value>,
) -> anyhow::Result<()> {
    let call_set = match args {
        Some(value) => CallSet::some_with_function_and_input(function_name, value),
        None => CallSet::some_with_function(function_name),
    };
    let signer = match contract.get_keys() {
        Some(keys) => Signer::Keys { keys },
        None => Signer::None,
    };

    let message_encode_params = ParamsOfEncodeMessage {
        abi: contract.abi.to_owned(),
        address: Some(contract.address.clone()),
        call_set,
        signer,
        deploy_set: None,
        processing_try_index: None,
        signature_id: None,
    };

    let _ = ton_client::processing::process_message(
        Arc::clone(client),
        ParamsOfProcessMessage {
            send_events: true,
            message_encode_params,
        },
        default_callback,
    )
    .await?;
    Ok(())
}

async fn default_callback(_pe: ProcessingEvent) {}
