use std::sync::Arc;

use alloy_network::eip2718::Encodable2718;
use alloy_primitives::{B256, U256};
use eth_trie::{EthTrie, MemoryDB, Trie};
use ethereum_types::H256;

use crate::{
    rpc::RpcProvider,
    tx_receipt::{ConsensusTxReceipt, RpcTxReceipt},
    Error,
};

/// Represents a handler for transactions Merkle Patricia Trie (MPT) operations,
/// including building the [`TxReceiptsMpt`] from transaction receipts and fetching proofs.
pub struct TxReceiptsMptHandler {
    /// Provides access to blockchain data via [`RpcProvider`].
    provider: RpcProvider,
    /// Optional MPT structure to hold transaction receipts.
    /// If `None`, the trie has not been built yet.
    trie: Option<TxReceiptsMpt>,
}

/// The [`TxReceiptsMpt`] struct encapsulates the MPT (Merkle Patricia Trie) specifically for transaction receipts,
/// including the trie structure itself, the [`ConsensusTxReceipt`] as elements, and the root hash.
pub struct TxReceiptsMpt {
    pub trie: EthTrie<MemoryDB>,
    elements: Vec<ConsensusTxReceipt>,
    root: B256,
}

impl TxReceiptsMptHandler {
    /// Creates a new [`TxReceiptsMptHandler`] with a given RPC provider URL.
    ///
    /// This does not initialize the trie yet.
    pub fn new(url: &str) -> Result<Self, Error> {
        let provider = RpcProvider::new(url);
        Ok(Self {
            provider,
            trie: None,
        })
    }

    /// Retrieves the index of a transaction within the trie based on its hash.
    ///
    /// Returns an error if the trie is not found or the transaction does not exist.
    pub async fn tx_hash_to_tx_index(&self, tx_hash: B256) -> Result<u64, Error> {
        let tx_index = self.provider.get_tx_index_by_hash(tx_hash).await?;
        Ok(tx_index)
    }

    /// Builds the receipt trie from a specific transaction hash.
    ///
    /// This fetches the block height for the transaction and delegates to [`build_tx_receipts_tree_from_block`].
    pub async fn build_tx_receipt_tree_from_tx_hash(&mut self, tx_hash: B256) -> Result<(), Error> {
        let height = self.provider.get_tx_block_height(tx_hash).await?;
        self.build_tx_receipts_tree_from_block(height).await?;
        Ok(())
    }

    /// Builds the transaction receipts trie from a given block number.
    ///
    /// This involves fetching the transactions for the block and [`build_trie`].
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

    /// Constructs the MPT from a vector of [`ConsensusTxReceipt`] and an expected root hash.
    ///
    /// Verifies the constructed trie's root against the expected root, returning an error if they do not match.
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

    /// Generates a proof for a transaction at a given index within the trie.
    pub fn get_proof(&mut self, tx_index: u64) -> Result<Vec<Vec<u8>>, Error> {
        let target_trie = self.trie.as_mut().ok_or(Error::TrieNotFound)?;
        let key = alloy_rlp::encode(U256::from(tx_index));
        let proof = target_trie.trie.get_proof(key.as_slice())?;

        Ok(proof)
    }

    /// Verifies a proof for a transaction at a given index against the stored trie.
    pub fn verify_proof(&self, tx_index: u64, proof: Vec<Vec<u8>>) -> Result<Vec<u8>, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        match target_trie.trie.verify_proof(
            H256::from_slice(target_trie.root.as_slice()),
            alloy_rlp::encode(U256::from(tx_index)).as_slice(),
            proof,
        ) {
            Ok(Some(result)) => Ok(result),
            _ => Err(Error::InvalidMPTProof),
        }
    }

    /// Retrieves a [`ConsensusTxReceipt`] by its index within the trie.
    pub fn get_tx_receipt(&self, tx_index: u64) -> Result<ConsensusTxReceipt, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        target_trie
            .elements
            .get(tx_index as usize)
            .ok_or(Error::TxNotFound)
            .cloned()
    }

    /// Retrieves all elements within the trie.
    pub fn get_elements(&self) -> Result<Vec<ConsensusTxReceipt>, Error> {
        let target_trie = self.trie.as_ref().ok_or(Error::TrieNotFound)?;
        Ok(target_trie.elements.to_vec())
    }

    /// Retrieves the root hash of the trie.
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

    const MAINNET_RPC_URL: &str = "https://mainnet.infura.io/v3/720000a7936b45c79d0868f70478e2e9";

    // Test cases
    // Byzantium: 4370000
    // EIP-2930(Berlin): 12244000
    // EIP-1559(London): 12965000
    // EIP-4844(Dencun): 19426589

    #[tokio::test]
    async fn test_tx_receipt_byzantium() {
        let target_tx_hash = B256::from(hex!(
            "1fcb1196d8a3bff0bcf13309d2d2bb1a23ae1ac13f5674c801be0ff9254d5ab5"
        ));

        let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).unwrap();
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
    async fn test_tx_receipt_2930() {
        let target_tx_hash = B256::from(hex!(
            "aa40dd75b18f375df1ae9a7f7de217fa3bc49b94db3c4da7b3974130990aefef"
        ));

        let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).unwrap();
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
    async fn test_tx_receipt_1559() {
        let target_tx_hash = B256::from(hex!(
            "2055b7e01304f87f9412cd44758cd248bc2da2dab95c97026064ffb084711735"
        ));

        let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).unwrap();
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
    async fn test_tx_receipt_4844() {
        // 4844 transaction
        let target_tx_hash = B256::from(hex!(
            "9c1fbda4f649ac806ab0faefbe94e1a60282eb374ead6aa01bac042f52b28a8c"
        ));

        let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).unwrap();
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
