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

pub struct MptHandler {
    provider: RpcProvider,
    trie: Option<Mpt>,
}

pub struct Mpt {
    trie: EthTrie<MemoryDB>,
    txs: Vec<ConsensusTx>,
    tx_root: B256,
}

impl MptHandler {
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
            .txs
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
        self.build_tx_tree(converted_txs, tx_root)?;
        Ok(())
    }

    pub fn build_tx_tree(
        &mut self,
        txs: Vec<ConsensusTx>,
        expected_root: B256,
    ) -> Result<(), Error> {
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

        let result_mpt = Mpt {
            trie,
            txs,
            tx_root: expected_root,
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
            H256::from_slice(target_trie.tx_root.as_slice()),
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
            .txs
            .get(tx_index as usize)
            .ok_or(Error::TxNotFound)
    }

    pub fn get_txs(&self) -> Result<&Vec<ConsensusTx>, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(&target_trie.txs)
    }

    pub fn get_tx_root(&self) -> Result<B256, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(target_trie.tx_root)
    }
}

pub struct TxReceiptsMptHandler {
    provider: RpcProvider,
    trie: Option<TxReceiptsMpt>,
}

pub struct TxReceiptsMpt {
    trie: EthTrie<MemoryDB>,
    txs: Vec<ConsensusTxReceipt>,
    tx_root: B256,
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
        Ok(tx_index as u64)
    }

    // pub async fn build_tx_tree_from_tx_hash(&mut self, tx_hash: B256) -> Result<(), Error> {
    //     let height = self.provider.get_tx_block_height(tx_hash).await?;
    //     self.build_tx_tree_from_block(height).await?;
    //     Ok(())
    // }

    pub async fn build_tx_tree_from_block(&mut self, block_number: u64) -> Result<(), Error> {
        let (txs, tx_receipt_root) = self
            .provider
            .get_block_transaction_receipts(block_number)
            .await?;

        let converted_tx_receipts: Vec<ConsensusTxReceipt> = txs
            .iter()
            .map(|tx_receipt| RpcTxReceipt(tx_receipt.clone()).try_into())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        self.build_tx_tree(converted_tx_receipts, tx_receipt_root)?;
        Ok(())
    }

    pub fn build_tx_tree(
        &mut self,
        txs: Vec<ConsensusTxReceipt>,
        expected_root: B256,
    ) -> Result<(), Error> {
        let memdb = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(memdb.clone());

        for (idx, tx) in txs.iter().enumerate() {
            let key = alloy_rlp::encode(U256::from(idx));
            let rlp = tx.0.encoded_2718();
            trie.insert(key.as_slice(), rlp.as_slice())?;
        }

        println!("Root: {:?}", trie.root_hash()?);
        println!("Expected Root: {:?}", expected_root.to_string());

        if trie.root_hash()?.as_bytes() != expected_root.as_slice() {
            return Err(Error::UnexpectedRoot);
        }

        let result_mpt = TxReceiptsMpt {
            trie,
            txs,
            tx_root: expected_root,
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
            H256::from_slice(target_trie.tx_root.as_slice()),
            alloy_rlp::encode(U256::from(tx_index)).as_slice(),
            proof,
        ) {
            Ok(Some(_)) => Ok(()),
            _ => Err(Error::InvalidMPTProof),
        }
    }

    pub fn get_tx_receipts(&self, tx_index: u64) -> Result<&ConsensusTxReceipt, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        target_trie
            .txs
            .get(tx_index as usize)
            .ok_or(Error::TxNotFound)
    }

    pub fn get_txs_receipts(&self) -> Result<&Vec<ConsensusTxReceipt>, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(&target_trie.txs)
    }

    pub fn get_tx_root(&self) -> Result<B256, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(target_trie.tx_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;
    use alloy_primitives::B256;

    const MAINNET_RPC_URL: &str = "https://mainnet.infura.io/v3/da91aac0e91048b3bf3be813262d43a6";

    #[tokio::test]
    async fn test_mpt_proof_from_tx_hash() {
        let mut mpt_handler = MptHandler::new(MAINNET_RPC_URL).await.unwrap();
        let tx_hash = B256::from(hex!(
            "ef1503cc8bd82da1552389183a097126bae21a889390a7be556b1f69d8c75c29"
        ));
        mpt_handler
            .build_tx_tree_from_tx_hash(tx_hash)
            .await
            .unwrap();

        let tx_index = mpt_handler.tx_hash_to_tx_index(tx_hash).unwrap();
        let proof = mpt_handler.get_proof(tx_index).unwrap();
        mpt_handler.verify_proof(tx_index, proof.clone()).unwrap();
    }

    #[tokio::test]
    async fn test_mpt_proof_from_number() {
        let mut mpt_handler = MptHandler::new(MAINNET_RPC_URL).await.unwrap();

        mpt_handler
            .build_tx_tree_from_block(19487818)
            .await
            .unwrap();
        let target_tx_hash = B256::from(hex!(
            "d1b736880e62738b04a1f277f099784bbdf548157d30d4edc41269553013ef13"
        ));
        let tx_index = mpt_handler.tx_hash_to_tx_index(target_tx_hash).unwrap();
        let proof = mpt_handler.get_proof(tx_index).unwrap();
        mpt_handler.verify_proof(tx_index, proof.clone()).unwrap();
    }

    #[tokio::test]
    async fn test_tx_receipt_mpt_proof_from_number() {
        let mut mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).await.unwrap();

        // Before EIP-2718
        mpt_handler
            .build_tx_tree_from_block(12244000)
            .await
            .unwrap();

        let target_tx_hash = B256::from(hex!(
            "0c14baf5342c882f46a4ad379a0ecc9fa582981dbf5b9bbba7d7ad50addec217"
        ));
        let tx_index = mpt_handler
            .tx_hash_to_tx_index(target_tx_hash)
            .await
            .unwrap();
        let proof = mpt_handler.get_proof(tx_index).unwrap();
        mpt_handler.verify_proof(tx_index, proof.clone()).unwrap();
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
