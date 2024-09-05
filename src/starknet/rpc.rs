use alloy_primitives::BlockNumber;
use serde_json::json;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockWithTxHashes, BlockWithTxs};

use crate::SnTrieError;

use super::tx_hash::calculate_transaction_hash;

pub struct RpcProvider {
    provider: reqwest::Client,
    url: url::Url,
}

impl RpcProvider {
    pub(crate) fn new(rpc_url: url::Url) -> Self {
        let provider = reqwest::Client::new();
        Self {
            provider,
            url: rpc_url,
        }
    }

    pub(crate) async fn get_block_transactions(
        &self,
        block_number: BlockNumber,
    ) -> Result<(), SnTrieError> {
        Ok(())
    }

    /// Fetches proof (account or storage) for a given block number
    async fn get_block_with_txs(&self, block_number: BlockNumber) -> Result<(), SnTrieError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "starknet_getBlockWithTxs",
            "params": {
                "block_id": {"block_number": block_number},
            }
        });

        let url = self.url.clone();

        let response = self.provider.post(url).json(&request).send().await.unwrap();
        let response_json =
            serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap()
                ["result"]
                .clone();

        let get_proof_output: BlockWithTxs<Felt> = serde_json::from_value(response_json).unwrap();
        let protocol = get_proof_output.block_header.starknet_version;
        let tx_final_hashes: Vec<Felt> = get_proof_output
            .transactions
            .iter()
            .map(|t| calculate_transaction_hash(t, &protocol))
            .collect();

        println!("tx_final:{:?}", tx_final_hashes);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::RpcProvider;

    const PATHFINDER_URL: &str = "https://pathfinder.sepolia.iosis.tech/";

    #[tokio::test]
    async fn test_get_block_with_txs() {
        let provider = RpcProvider::new(url::Url::from_str(PATHFINDER_URL).unwrap());
        // 0.13.2 - invoke, declare,deploy_account
        // provider.get_block_with_txs(124358).await;
        // // 0.13.2 - invoke, l1_handler
        // provider.get_block_with_txs(124015).await;
        // // 0.13.2 - invoke
        // provider.get_block_with_txs(99708).await;

        // provider.get_block_with_txs(71224).await;

        // provider.get_block_with_txs(71311).await;

        // provider.get_block_with_txs(70015).await;

        // provider.get_block_with_txs(50304).await;
        // provider.get_block_with_txs(51190).await;

        // provider.get_block_with_txs(34999).await;

        // provider.get_block_with_txs(35000).await;
        provider.get_block_with_txs(7).await;
    }
}
