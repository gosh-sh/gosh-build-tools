use std::sync::Arc;
use ton_client::net::{ParamsOfQueryCollection, query_collection};
use crate::blockchain::contract::Contract;
use crate::blockchain::ever_client::EverClient;

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
        Arc::clone(context),
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
    let AccountBoc { boc, .. } = serde_json::from_value(query[0].clone())?;
}