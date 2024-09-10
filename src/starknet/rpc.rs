use alloy_primitives::BlockNumber;
use serde_json::{json, Value};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::BlockWithTxs;

use crate::SnTrieError;

pub struct RpcProvider<'a> {
    url: &'a str,
    gateway_url: &'a str,
}

impl<'a> RpcProvider<'a> {
    pub(crate) fn new(rpc_url: &'a str, gateway_url: &'a str) -> Self {
        Self {
            url: rpc_url,
            gateway_url,
        }
    }

    pub(crate) async fn get_block_transactions(
        &self,
        block_number: BlockNumber,
    ) -> Result<(BlockWithTxs<Felt>, String), SnTrieError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "starknet_getBlockWithTxs",
            "params": {
                "block_id": {"block_number": block_number},
            }
        });

        let url = self.url;
        let provider = reqwest::Client::new();
        let response = provider.post(url).json(&request).send().await.unwrap();
        let response_json =
            serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap()
                ["result"]
                .clone();

        let get_proof_output: BlockWithTxs<Felt> = serde_json::from_value(response_json).unwrap();
        let gateway = GatewayProvider::new(self.gateway_url.to_string());
        let transaction_commitment = gateway.get_tx_commit(block_number).await.unwrap();

        Ok((get_proof_output, transaction_commitment))
    }
}

pub const GATEWAY_URL: &str = "https://alpha-sepolia.starknet.io";

pub struct GatewayProvider {
    base_url: String,
}

impl GatewayProvider {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    async fn get_tx_commit(&self, block_number: u64) -> Result<String, SnTrieError> {
        let url = format!(
            "{}/feeder_gateway/get_block?blockNumber={}",
            self.base_url, block_number
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await.unwrap();

        if response.status().is_success() {
            let block_data: Value = response.json().await.unwrap();
            let block_data: &Value = &block_data["transaction_commitment"];
            Ok(block_data.to_string())
        } else {
            Err(SnTrieError::GatewayError(response.status().as_u16()))
        }
    }
}
