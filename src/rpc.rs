use crate::Error;
use alloy_network::Ethereum;
use alloy_primitives::B256;
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_client::RpcClient;
use alloy_rpc_types::{BlockTransactions, Transaction, TransactionReceipt};
use alloy_transport::{RpcError, TransportErrorKind};
use alloy_transport_http::Http;
use reqwest::Client;

pub(crate) struct RpcProvider {
    provider: RootProvider<Ethereum, Http<Client>>,
}

impl RpcProvider {
    pub(crate) fn new(url: &str) -> Self {
        let http = Http::<Client>::new(url.to_string().parse().unwrap());
        let provider = ProviderBuilder::<_, Ethereum>::new()
            .provider(RootProvider::new(RpcClient::new(http, true)));
        Self { provider }
    }

    pub(crate) async fn get_block_transactions(
        &self,
        block_number: u64,
    ) -> Result<(Vec<Transaction>, B256), Error> {
        let block = self
            .provider
            .get_block(block_number.into(), true)
            .await?
            .ok_or_else(|| Error::BlockNotFound)?;

        let txs = match block.transactions {
            BlockTransactions::Full(txs) => txs,
            _ => return Err(Error::TxNotFound),
        };

        Ok((txs, block.header.transactions_root))
    }

    pub(crate) async fn get_block_transaction_receipts(
        &self,
        block_number: u64,
    ) -> Result<(Vec<TransactionReceipt>, B256), Error> {
        let block = self
            .provider
            .get_block(block_number.into(), true)
            .await?
            .ok_or_else(|| Error::BlockNotFound)?;

        let tx_receipts = self
            .provider
            .get_block_receipts(block_number.into())
            .await?
            .ok_or_else(|| Error::BlockNotFound)?;

        Ok((tx_receipts, block.header.receipts_root))
    }

    pub(crate) async fn get_tx_block_height(&self, tx_hash: B256) -> Result<u64, Error> {
        let tx = self.provider.get_transaction_by_hash(tx_hash).await?;

        let height: u64 = match tx.block_number {
            Some(height) => height.try_into().map_err(|_| Error::TxNotFound)?,
            None => return Err(Error::TxNotFound),
        };

        Ok(height)
    }
}

impl From<RpcError<TransportErrorKind>> for Error {
    fn from(err: RpcError<TransportErrorKind>) -> Self {
        Error::RPC(err)
    }
}
