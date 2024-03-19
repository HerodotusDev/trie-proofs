use crate::Error;
use alloy_network::Ethereum;
use alloy_primitives::B256;
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_client::RpcClient;
use alloy_rpc_types::{BlockTransactions, Transaction};
use alloy_transport_http::Http;
use reqwest::Client;

pub struct RpcProvider {
    provider: RootProvider<Ethereum, Http<Client>>,
}

pub struct TransactionsWithRoot {
    pub transactions: Vec<Transaction>,
    pub root: B256,
}

impl RpcProvider {
    pub fn new() -> Self {
        let http = Http::<Client>::new(
            "https://mainnet.infura.io/v3/da91aac0e91048b3bf3be813262d43a6"
                .to_string()
                .parse()
                .unwrap(),
        );
        let provider = ProviderBuilder::<_, Ethereum>::new()
            .provider(RootProvider::new(RpcClient::new(http, true)));
        Self { provider }
    }

    pub async fn get_block_transactions(
        &self,
        block_number: u64,
    ) -> Result<TransactionsWithRoot, Error> {
        let block = self
            .provider
            .get_block(block_number.into(), true)
            .await
            .map_err(|err| Error::RPC(err.to_string()))?
            .ok_or_else(|| Error::RPC("Unable to retrieve block".to_string()))?;

        match block.transactions {
            BlockTransactions::Full(txs) => Ok(TransactionsWithRoot {
                transactions: txs,
                root: block.header.transactions_root,
            }),
            _ => Err(Error::RPC("Unable to retrieve block txs".to_string())),
        }
    }

    pub async fn get_tx_block_height(&self, tx_hash: B256) -> Result<u64, Error> {
        let tx = self
            .provider
            .get_transaction_by_hash(tx_hash)
            .await
            .map_err(|err| Error::RPC(err.to_string()))?;

        let height: u64 = match tx.block_number {
            Some(height) => height.try_into().map_err(|_| Error::TxNotFound)?,
            None => return Err(Error::TxNotFound),
        };

        Ok(height)
    }
}
