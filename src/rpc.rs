use alloy_network::Ethereum;
use alloy_primitives::{B256, U256};
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_client::RpcClient;
use alloy_rpc_types::{BlockTransactions, Transaction};
use alloy_transport::{RpcError, TransportError, TransportErrorKind};
use alloy_transport_http::Http;
use reqwest::Client;
use crate::Error;

pub struct RpcProvider {
    provider: RootProvider<Ethereum, Http<Client>>,
}

impl RpcProvider {
    pub fn new() -> Self {
        let http = Http::<Client>::new("https://mainnet.infura.io/v3/da91aac0e91048b3bf3be813262d43a6".to_string().parse().unwrap());
        let provider = ProviderBuilder::<_, Ethereum>::new()
            .provider(RootProvider::new(RpcClient::new(http, true)));
        Self {
            provider
        }
    }

    pub async fn get_block_transactions(&self, block_number: u64, tx_hash: B256) -> Result<(Vec<Transaction>, B256, u64), Error> {
        let block = self.provider
            .get_block(block_number.into(), true)
            .await?
            .ok_or_else(|| Error::BlockNotFound)?;

        let txs = match block.transactions {
            BlockTransactions::Full(txs) => txs,
            _ => return Err(Error::TxNotFound)
        };

        let tx_index = txs.iter().position(|tx| tx.hash == tx_hash).ok_or(Error::TxNotFound)?;

        Ok((txs, block.header.transactions_root, tx_index as u64))
    }

    pub async fn get_tx_block_height(&self, tx_hash: B256) -> Result<u64, Error> {
        let tx = self.provider
            .get_transaction_by_hash(tx_hash.into())
            .await?;

        let height: u64 = match tx.block_number {
            Some(height) => height.try_into().map_err(|_| Error::TxNotFound)?,
            None => return Err(Error::TxNotFound)
        };

        Ok(height)
    }
}

impl From<RpcError<TransportErrorKind>> for Error {
    fn from(err: RpcError<TransportErrorKind>) -> Self {
        Error::RPC(err)
    }
}