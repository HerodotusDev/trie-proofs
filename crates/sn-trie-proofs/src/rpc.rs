use serde_json::{json, Value};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockWithReceipts, BlockWithTxs};

use crate::error::SnTrieError;

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
        block_number: u64,
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

    pub(crate) async fn get_block_transactions_receipts(
        &self,
        block_number: u64,
    ) -> Result<(BlockWithReceipts<Felt>, Vec<u64>, String), SnTrieError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "starknet_getBlockWithReceipts",
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

        let get_proof_output: BlockWithReceipts<Felt> =
            serde_json::from_value(response_json).unwrap();
        let gateway = GatewayProvider::new(self.gateway_url.to_string());
        let (l1_gas_vec, receipt_commitment) = gateway.get_l1_gas(block_number).await.unwrap();

        Ok((get_proof_output, l1_gas_vec, receipt_commitment))
    }
}

pub const GATEWAY_URL: &str = "https://alpha-sepolia.starknet.io";

pub struct GatewayProvider {
    base_url: String,
}

impl GatewayProvider {
    pub fn new(base_url: impl AsRef<str>) -> Self {
        Self {
            base_url: base_url.as_ref().to_string(),
        }
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

    /// Note: This method is only available after 0.13.2
    async fn get_l1_gas(&self, block_number: u64) -> Result<(Vec<u64>, String), SnTrieError> {
        let url = format!(
            "{}/feeder_gateway/get_block?blockNumber={}",
            self.base_url, block_number
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await.unwrap();

        if response.status().is_success() {
            let block_data: Value = response.json().await.unwrap();

            println!("{:?}", block_data);
            let receipt_commitment = block_data["receipt_commitment"].to_string();
            let transaction_receipts = block_data["transaction_receipts"].as_array().unwrap();

            let gas_consumed: Vec<u64> = transaction_receipts
                .iter()
                .filter_map(|receipt| {
                    receipt["execution_resources"]["total_gas_consumed"]["l1_gas"].as_u64()
                })
                .collect();

            Ok((gas_consumed, receipt_commitment))
        } else {
            Err(SnTrieError::GatewayError(response.status().as_u16()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const PATHFINDER_URL: &str = "https://pathfinder.sepolia.iosis.tech/";
    const GATEWAY_URL: &str = "https://alpha-sepolia.starknet.io";

    #[tokio::test]
    async fn test_gateway() {
        let gateway = GatewayProvider::new(GATEWAY_URL);
        let total_gas_consumed = gateway.get_l1_gas(99708).await;
        println!("{:?}", total_gas_consumed);
    }

    #[tokio::test]
    async fn test_rpc() {
        let rpc = RpcProvider::new(PATHFINDER_URL, GATEWAY_URL);
        let (receipts, l1_gas, _) = rpc.get_block_transactions_receipts(99708).await.unwrap();
        assert_eq!(receipts.transactions.len(), l1_gas.len())
    }
}
