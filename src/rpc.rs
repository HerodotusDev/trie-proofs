use crate::EthTrieError;
use alloy::network::Ethereum;
use alloy::primitives::B256;
use alloy::providers::{Provider, RootProvider};

use alloy::rpc::types::{BlockTransactions, Transaction, TransactionReceipt};
use alloy::transports::http::{Client, Http};
use alloy::transports::{RpcError, TransportErrorKind};

pub(crate) struct RpcProvider {
    provider: RootProvider<Http<Client>, Ethereum>,
}

impl RpcProvider {
    pub(crate) fn new(rpc_url: url::Url) -> Self {
        let provider = RootProvider::new_http(rpc_url);
        Self { provider }
    }

    pub(crate) async fn get_block_transactions(
        &self,
        block_number: u64,
    ) -> Result<(Vec<Transaction>, B256), EthTrieError> {
        let block = self
            .provider
            .get_block(
                block_number.into(),
                alloy::rpc::types::BlockTransactionsKind::Full,
            )
            .await?
            .ok_or_else(|| EthTrieError::BlockNotFound)?;

        let txs = match block.transactions {
            BlockTransactions::Full(txs) => txs,
            _ => return Err(EthTrieError::TxNotFound),
        };

        Ok((txs, block.header.transactions_root))
    }

    pub(crate) async fn get_block_transaction_receipts(
        &self,
        block_number: u64,
    ) -> Result<(Vec<TransactionReceipt>, B256), EthTrieError> {
        let block = self
            .provider
            .get_block(
                block_number.into(),
                alloy::rpc::types::BlockTransactionsKind::Full,
            )
            .await?
            .ok_or_else(|| EthTrieError::BlockNotFound)?;

        let tx_receipts = self
            .provider
            .get_block_receipts(block_number.into())
            .await?
            .ok_or_else(|| EthTrieError::BlockNotFound)?;

        Ok((tx_receipts, block.header.receipts_root))
    }

    pub(crate) async fn get_tx_index_by_hash(&self, tx_hash: B256) -> Result<u64, EthTrieError> {
        let tx = self
            .provider
            .get_transaction_by_hash(tx_hash)
            .await?
            .expect("tx not found");

        let index: u64 = match tx.transaction_index {
            Some(index) => index,
            None => return Err(EthTrieError::TxNotFound),
        };

        Ok(index)
    }

    pub(crate) async fn get_tx_block_height(&self, tx_hash: B256) -> Result<u64, EthTrieError> {
        let tx = self
            .provider
            .get_transaction_by_hash(tx_hash)
            .await?
            .expect("tx not found");

        let height: u64 = match tx.block_number {
            Some(height) => height,
            None => return Err(EthTrieError::TxNotFound),
        };

        Ok(height)
    }
}

impl From<RpcError<TransportErrorKind>> for EthTrieError {
    fn from(err: RpcError<TransportErrorKind>) -> Self {
        EthTrieError::RPC(err)
    }
}
