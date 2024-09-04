use alloy_primitives::BlockNumber;
use serde_json::json;
use url::Url;

use crate::SnTrieError;

use super::tx::BlockWithTxsOutput;

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
    async fn get_block_with_txs(
        &self,
        url: Url,
        block_number: BlockNumber,
    ) -> Result<(), SnTrieError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "starknet_getBlockWithTxs",
            "params": {
                "block_id": {"block_number": block_number},
            }
        });

        let response = self.provider.post(url).json(&request).send().await.unwrap();
        let response_json =
            serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap()
                ["result"]
                .clone();
        let get_proof_output: BlockWithTxsOutput = serde_json::from_value(response_json).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    fn test_get_block_with_txs() {}
}
