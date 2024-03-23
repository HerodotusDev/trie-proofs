mod rpc;
mod tx;
mod tx_receipt;

use crate::tx::ConsensusTx;

use alloy_primitives::B256;

use alloy_transport::{RpcError, TransportErrorKind};
use eth_trie::{EthTrie, MemoryDB, TrieError};

use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::U256;
use eth_trie::Trie;
use ethereum_types::H256;
use rpc::RpcProvider;
use std::sync::Arc;
use tx::RpcTx;
use tx_receipt::{ConsensusTxReceipt, RpcTxReceipt};

pub struct TxsMptHandler {
    provider: RpcProvider,
    trie: Option<TxsMpt>,
}

pub struct TxsMpt {
    trie: EthTrie<MemoryDB>,
    elements: Vec<ConsensusTx>,
    root: B256,
}

impl TxsMptHandler {
    pub async fn new(url: &str) -> Result<Self, Error> {
        let provider = RpcProvider::new(url);
        Ok(Self {
            provider,
            trie: None,
        })
    }

    pub fn tx_hash_to_tx_index(&self, tx_hash: B256) -> Result<u64, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        let tx_index = target_trie
            .elements
            .iter()
            .position(|tx| tx.0.trie_hash() == tx_hash)
            .ok_or(Error::TxNotFound)?;
        Ok(tx_index as u64)
    }

    pub async fn build_tx_tree_from_tx_hash(&mut self, tx_hash: B256) -> Result<(), Error> {
        let height = self.provider.get_tx_block_height(tx_hash).await?;
        self.build_tx_tree_from_block(height).await?;
        Ok(())
    }

    pub async fn build_tx_tree_from_block(&mut self, block_number: u64) -> Result<(), Error> {
        let (txs, tx_root) = self.provider.get_block_transactions(block_number).await?;
        let converted_txs: Vec<ConsensusTx> = txs
            .iter()
            .map(|tx| RpcTx(tx.clone()).try_into())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        self.build_trie(converted_txs, tx_root)?;
        Ok(())
    }

    pub fn build_trie(&mut self, txs: Vec<ConsensusTx>, expected_root: B256) -> Result<(), Error> {
        let memdb = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(memdb.clone());

        for (idx, tx) in txs.iter().enumerate() {
            let key = alloy_rlp::encode(U256::from(idx));
            let rlp = tx.0.encoded_2718();
            trie.insert(key.as_slice(), rlp.as_slice())?;
        }

        if trie.root_hash()?.as_bytes() != expected_root.as_slice() {
            return Err(Error::UnexpectedRoot);
        }

        let result_mpt = TxsMpt {
            trie,
            elements: txs,
            root: expected_root,
        };

        self.trie = Some(result_mpt);
        Ok(())
    }

    pub fn get_proof(&mut self, tx_index: u64) -> Result<Vec<Vec<u8>>, Error> {
        let target_trie = self.trie.as_mut().ok_or(Error::TrieNotFound)?;
        let key = alloy_rlp::encode(U256::from(tx_index));
        let proof = target_trie.trie.get_proof(key.as_slice())?;

        Ok(proof)
    }

    pub fn verify_proof(&self, tx_index: u64, proof: Vec<Vec<u8>>) -> Result<(), Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        match target_trie.trie.verify_proof(
            H256::from_slice(target_trie.root.as_slice()),
            alloy_rlp::encode(U256::from(tx_index)).as_slice(),
            proof,
        ) {
            Ok(Some(_)) => Ok(()),
            _ => Err(Error::InvalidMPTProof),
        }
    }

    pub fn get_tx(&self, tx_index: u64) -> Result<&ConsensusTx, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        target_trie
            .elements
            .get(tx_index as usize)
            .ok_or(Error::TxNotFound)
    }

    pub fn get_elements(&self) -> Result<&Vec<ConsensusTx>, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(&target_trie.elements)
    }

    pub fn get_root(&self) -> Result<B256, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(target_trie.root)
    }
}

pub struct TxReceiptsMptHandler {
    provider: RpcProvider,
    trie: Option<TxReceiptsMpt>,
}

pub struct TxReceiptsMpt {
    trie: EthTrie<MemoryDB>,
    elements: Vec<ConsensusTxReceipt>,
    root: B256,
}

impl TxReceiptsMptHandler {
    pub async fn new(url: &str) -> Result<Self, Error> {
        let provider = RpcProvider::new(url);
        Ok(Self {
            provider,
            trie: None,
        })
    }

    pub async fn tx_hash_to_tx_index(&self, tx_hash: B256) -> Result<u64, Error> {
        let tx_index = self.provider.get_tx_index_by_hash(tx_hash).await?;
        Ok(tx_index)
    }

    pub async fn build_tx_receipts_tree_from_block(
        &mut self,
        block_number: u64,
    ) -> Result<(), Error> {
        let (txs, tx_receipt_root) = self
            .provider
            .get_block_transaction_receipts(block_number)
            .await?;

        let converted_tx_receipts: Vec<ConsensusTxReceipt> = txs
            .iter()
            .map(|tx_receipt| RpcTxReceipt(tx_receipt.clone()).try_into())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        self.build_trie(converted_tx_receipts, tx_receipt_root)?;
        Ok(())
    }

    pub fn build_trie(
        &mut self,
        tx_receipts: Vec<ConsensusTxReceipt>,
        expected_root: B256,
    ) -> Result<(), Error> {
        let memdb = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(memdb.clone());

        for (idx, tx) in tx_receipts.iter().enumerate() {
            let key = alloy_rlp::encode(U256::from(idx));
            let rlp = tx.0.encoded_2718();
            trie.insert(key.as_slice(), rlp.as_slice())?;
        }

        if trie.root_hash()?.as_bytes() != expected_root.as_slice() {
            return Err(Error::UnexpectedRoot);
        }

        let result_mpt = TxReceiptsMpt {
            trie,
            elements: tx_receipts,
            root: expected_root,
        };

        self.trie = Some(result_mpt);
        Ok(())
    }

    pub fn get_proof(&mut self, tx_index: u64) -> Result<Vec<Vec<u8>>, Error> {
        let target_trie = self.trie.as_mut().ok_or(Error::TrieNotFound)?;
        let key = alloy_rlp::encode(U256::from(tx_index));
        let proof = target_trie.trie.get_proof(key.as_slice())?;

        Ok(proof)
    }

    pub fn verify_proof(&self, tx_index: u64, proof: Vec<Vec<u8>>) -> Result<(), Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        match target_trie.trie.verify_proof(
            H256::from_slice(target_trie.root.as_slice()),
            alloy_rlp::encode(U256::from(tx_index)).as_slice(),
            proof,
        ) {
            Ok(Some(_)) => Ok(()),
            _ => Err(Error::InvalidMPTProof),
        }
    }

    pub fn get_tx_receipt(&self, tx_index: u64) -> Result<&ConsensusTxReceipt, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        target_trie
            .elements
            .get(tx_index as usize)
            .ok_or(Error::TxNotFound)
    }

    pub fn get_elements(&self) -> Result<&Vec<ConsensusTxReceipt>, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(&target_trie.elements)
    }

    pub fn get_root(&self) -> Result<B256, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(target_trie.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;
    use alloy_primitives::B256;

    const MAINNET_RPC_URL: &str = "https://mainnet.infura.io/v3/da91aac0e91048b3bf3be813262d43a6";

    // Test cases
    // Frontier: 46147 (receipts not available)
    // Byzantium: 4370000
    // EIP-2930(Berlin): 12244000
    // EIP-1559(London): 12965000
    // EIP-4844(Dencun): 19426589

    #[tokio::test]
    async fn test_tx_mpt_frontier() {
        let target_tx_hash = B256::from(hex!(
            "5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060"
        ));

        let mut txs_mpt_handler = TxsMptHandler::new(MAINNET_RPC_URL).await.unwrap();

        txs_mpt_handler
            .build_tx_tree_from_block(46147)
            .await
            .unwrap();

        let tx_index = txs_mpt_handler.tx_hash_to_tx_index(target_tx_hash).unwrap();
        let proof = txs_mpt_handler.get_proof(tx_index).unwrap();
        txs_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }

    #[tokio::test]
    async fn test_tx_mpt_byzantium() {
        let target_tx_hash = B256::from(hex!(
            "1fcb1196d8a3bff0bcf13309d2d2bb1a23ae1ac13f5674c801be0ff9254d5ab5"
        ));

        let mut txs_mpt_handler = TxsMptHandler::new(MAINNET_RPC_URL).await.unwrap();

        txs_mpt_handler
            .build_tx_tree_from_block(4370000)
            .await
            .unwrap();

        let tx_index = txs_mpt_handler.tx_hash_to_tx_index(target_tx_hash).unwrap();
        let proof = txs_mpt_handler.get_proof(tx_index).unwrap();
        txs_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }

    #[tokio::test]
    async fn test_tx_receipt_byzantium() {
        let target_tx_hash = B256::from(hex!(
            "1fcb1196d8a3bff0bcf13309d2d2bb1a23ae1ac13f5674c801be0ff9254d5ab5"
        ));

        let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).await.unwrap();
        tx_receipts_mpt_handler
            .build_tx_receipts_tree_from_block(4370000)
            .await
            .unwrap();

        let tx_index = tx_receipts_mpt_handler
            .tx_hash_to_tx_index(target_tx_hash)
            .await
            .unwrap();
        let proof = tx_receipts_mpt_handler.get_proof(tx_index).unwrap();
        tx_receipts_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }

    #[tokio::test]
    async fn test_tx_mpt_2930() {
        let target_tx_hash = B256::from(hex!(
            "aa40dd75b18f375df1ae9a7f7de217fa3bc49b94db3c4da7b3974130990aefef"
        ));

        let mut txs_mpt_handler = TxsMptHandler::new(MAINNET_RPC_URL).await.unwrap();

        txs_mpt_handler
            .build_tx_tree_from_block(12244000)
            .await
            .unwrap();

        let tx_index = txs_mpt_handler.tx_hash_to_tx_index(target_tx_hash).unwrap();
        let proof = txs_mpt_handler.get_proof(tx_index).unwrap();
        txs_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }

    #[tokio::test]
    async fn test_tx_receipt_2930() {
        let target_tx_hash = B256::from(hex!(
            "aa40dd75b18f375df1ae9a7f7de217fa3bc49b94db3c4da7b3974130990aefef"
        ));

        let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).await.unwrap();
        tx_receipts_mpt_handler
            .build_tx_receipts_tree_from_block(12244000)
            .await
            .unwrap();

        let tx_index = tx_receipts_mpt_handler
            .tx_hash_to_tx_index(target_tx_hash)
            .await
            .unwrap();
        let proof = tx_receipts_mpt_handler.get_proof(tx_index).unwrap();
        tx_receipts_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }

    #[tokio::test]
    async fn test_tx_mpt_1559() {
        let target_tx_hash = B256::from(hex!(
            "2055b7e01304f87f9412cd44758cd248bc2da2dab95c97026064ffb084711735"
        ));

        let mut txs_mpt_handler = TxsMptHandler::new(MAINNET_RPC_URL).await.unwrap();

        txs_mpt_handler
            .build_tx_tree_from_block(12965000)
            .await
            .unwrap();

        let tx_index = txs_mpt_handler.tx_hash_to_tx_index(target_tx_hash).unwrap();
        let proof = txs_mpt_handler.get_proof(tx_index).unwrap();
        txs_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }

    #[tokio::test]
    async fn test_tx_receipt_1559() {
        let target_tx_hash = B256::from(hex!(
            "2055b7e01304f87f9412cd44758cd248bc2da2dab95c97026064ffb084711735"
        ));

        let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).await.unwrap();
        tx_receipts_mpt_handler
            .build_tx_receipts_tree_from_block(12965000)
            .await
            .unwrap();

        let tx_index = tx_receipts_mpt_handler
            .tx_hash_to_tx_index(target_tx_hash)
            .await
            .unwrap();
        let proof = tx_receipts_mpt_handler.get_proof(tx_index).unwrap();
        tx_receipts_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }

    #[tokio::test]
    async fn test_tx_mpt_4844() {
        // 4844 transaction
        let target_tx_hash = B256::from(hex!(
            "9c1fbda4f649ac806ab0faefbe94e1a60282eb374ead6aa01bac042f52b28a8c"
        ));

        let mut txs_mpt_handler = TxsMptHandler::new(MAINNET_RPC_URL).await.unwrap();

        txs_mpt_handler
            .build_tx_tree_from_block(19426589)
            .await
            .unwrap();

        let tx_index = txs_mpt_handler.tx_hash_to_tx_index(target_tx_hash).unwrap();
        let proof = txs_mpt_handler.get_proof(tx_index).unwrap();
        txs_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }

    #[tokio::test]
    async fn test_tx_receipt_4844() {
        // 4844 transaction
        let target_tx_hash = B256::from(hex!(
            "9c1fbda4f649ac806ab0faefbe94e1a60282eb374ead6aa01bac042f52b28a8c"
        ));

        let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).await.unwrap();
        tx_receipts_mpt_handler
            .build_tx_receipts_tree_from_block(19426589)
            .await
            .unwrap();

        let tx_index = tx_receipts_mpt_handler
            .tx_hash_to_tx_index(target_tx_hash)
            .await
            .unwrap();
        let proof = tx_receipts_mpt_handler.get_proof(tx_index).unwrap();
        tx_receipts_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();
    }
}

#[derive(Debug)]
pub enum Error {
    Trie(TrieError),
    Rlp(alloy_rlp::Error),
    RPC(RpcError<TransportErrorKind>),
    TxNotFound,
    BlockNotFound,
    InvalidTxVersion,
    ConversionError(Field),
    UnexpectedRoot,
    InvalidMPTProof,
    TrieNotFound,
}

#[derive(Debug)]
pub enum Field {
    ChainId,
    Nonce,
    GasPrice,
    GasLimit,
    Input,
    AccessList,
    MaxFeePerGas,
    MaxPriorityFeePerGas,
    MaxFeePerBlobGas,
    Signature,
}

impl From<TrieError> for Error {
    fn from(err: TrieError) -> Self {
        Error::Trie(err)
    }
}
