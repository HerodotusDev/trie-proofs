use crate::error::SnTrieError;
use crate::tx_receipt_hash::calculate_receipt_hash;
use sn_merkle_trie::conversion::from_u64_to_bits;
use sn_merkle_trie::node::TrieNode;
use sn_merkle_trie::transaction::TransactionMerkleTree;
use sn_merkle_trie::{Membership, MerkleTree};
use starknet_types_core::hash::{Poseidon, StarkHash};
use starknet_types_core::{felt::Felt, hash::Pedersen};

use super::rpc::RpcProvider;
use super::rpc::GATEWAY_URL;

/// Note: only implemented after 0.13.2 version
pub struct TxReceiptsMptHandler<'a> {
    provider: RpcProvider<'a>,
    trie: Option<TxReceiptsMpt>,
}

pub struct TxReceiptsMpt {
    pub trie: TransactionMerkleTree,
    pub elements: Vec<Felt>,
    root: Felt,
    root_idx: u64,
}

impl<'a> TxReceiptsMptHandler<'a> {
    pub fn new(rpc_url: &'a str) -> Result<Self, SnTrieError> {
        let provider = RpcProvider::new(rpc_url, GATEWAY_URL);
        Ok(Self {
            provider,
            trie: None,
        })
    }

    /// Build
    pub async fn build_tx_receipts_tree_from_block(
        &mut self,
        block_number: u64,
    ) -> Result<(), SnTrieError> {
        let (txs, vec_l1_gas, expected_commit) = self
            .provider
            .get_block_transactions_receipts(block_number)
            .await
            .expect("rpc fetch failed");
        let protocol = txs.block_header.starknet_version;

        if protocol.as_str() < "0.13.2" {
            return Err(SnTrieError::UnsupportedProtocol);
        }

        let tx_final_hashes: Vec<Felt> = txs
            .transactions
            .iter()
            .zip(vec_l1_gas.iter())
            .map(|(t, &l1_gas)| calculate_receipt_hash(t, l1_gas))
            .collect();

        self.build_trie(tx_final_hashes, &expected_commit, &protocol)?;
        Ok(())
    }

    pub fn build_trie(
        &mut self,
        txs: Vec<Felt>,
        expected_commit: &str,
        protocol: &str,
    ) -> Result<(), SnTrieError> {
        let trie = if protocol >= "0.13.2" {
            self.build_trie_generic::<Poseidon>(txs, expected_commit)?
        } else {
            self.build_trie_generic::<Pedersen>(txs, expected_commit)?
        };

        self.trie = Some(trie);
        Ok(())
    }

    fn build_trie_generic<H: StarkHash + 'static>(
        &self,
        txs: Vec<Felt>,
        expected_commit: &str,
    ) -> Result<TxReceiptsMpt, SnTrieError> {
        let mut tree = if std::any::TypeId::of::<H>() == std::any::TypeId::of::<Poseidon>() {
            TransactionMerkleTree::Poseidon(MerkleTree::default())
        } else {
            TransactionMerkleTree::Pedersen(MerkleTree::default())
        };

        for (idx, hash) in txs.clone().into_iter().enumerate() {
            let idx: u64 = idx.try_into().unwrap();
            let key = from_u64_to_bits(idx);
            tree.set(key, hash).expect("set failed");
        }

        let (root, root_idx) = tree.commit().expect("commit failed");

        let cleaned_expected_commit = expected_commit.trim_matches('"').to_string();
        assert_eq!(cleaned_expected_commit, root.to_hex_string());
        if cleaned_expected_commit != root.to_hex_string() {
            return Err(SnTrieError::InvalidCommitment);
        }

        Ok(TxReceiptsMpt {
            trie: tree,
            elements: txs,
            root,
            root_idx,
        })
    }

    pub fn get_proof(&self, tx_index: u64) -> Result<Vec<TrieNode>, SnTrieError> {
        let trie = self.trie.as_ref().ok_or(SnTrieError::TrieNotFound)?;
        let root_idx = trie.root_idx;
        let proof = trie
            .trie
            .get_proof(root_idx, from_u64_to_bits(tx_index))
            .unwrap()
            .ok_or(SnTrieError::TrieNotFound)?;
        Ok(proof)
    }

    pub fn verify_proof(
        &self,
        tx_index: u64,
        proof: Vec<TrieNode>,
    ) -> Result<Membership, SnTrieError> {
        let trie = self.trie.as_ref().ok_or(SnTrieError::TrieNotFound)?;
        let value = trie
            .elements
            .get(tx_index as usize)
            .ok_or(SnTrieError::InvalidTxIndex)?;

        let result = trie
            .trie
            .verify_proof(trie.root, &from_u64_to_bits(tx_index), *value, &proof)
            .ok_or(SnTrieError::VerificationError)?;
        Ok(result)
    }

    pub fn get_root_idx(&self) -> Result<u64, SnTrieError> {
        let trie = self.trie.as_ref().ok_or(SnTrieError::TrieNotFound)?;
        let root_idx = trie.root_idx;
        Ok(root_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATHFINDER_URL: &str = "https://pathfinder.sepolia.iosis.tech/";

    #[tokio::test]
    async fn test_build_tx_tree_from_block_4() {
        let mut handler = TxReceiptsMptHandler::new(PATHFINDER_URL).unwrap();
        //  # 0.13.2
        let block_number = 99708;
        handler
            .build_tx_receipts_tree_from_block(block_number)
            .await
            .unwrap();

        let proof = handler.get_proof(0).unwrap();
        let membership: Membership = handler.verify_proof(0, proof).unwrap();

        assert!(membership.is_member());

        let proof = handler.get_proof(1).unwrap();
        let membership: Membership = handler.verify_proof(1, proof).unwrap();

        assert!(membership.is_member());

        let proof = handler.get_proof(2).unwrap();
        let membership: Membership = handler.verify_proof(2, proof).unwrap();

        assert!(membership.is_member());

        let proof = handler.get_proof(3).unwrap();
        let membership: Membership = handler.verify_proof(3, proof).unwrap();

        assert!(membership.is_member());
    }
}
