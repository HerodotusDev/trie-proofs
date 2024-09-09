use sn_trie::conversion::from_felt_to_bits;
use sn_trie::{node::TrieNode, storage::memory::InMememoryStorage};
use sn_trie::{Membership, MerkleTree};
use starknet_types_core::{felt::Felt, hash::Pedersen};

use crate::SnTrieError;

use super::{rpc::RpcProvider, tx_hash::calculate_transaction_hash};

pub struct TxsMptHandler<'a> {
    provider: RpcProvider<'a>,
    trie: Option<TxsMpt>,
}

pub struct TxsMpt {
    pub trie: MerkleTree<Pedersen, InMememoryStorage, 64>,
    pub elements: Vec<Felt>,
    root: Felt,
    root_idx: u64,
}

impl<'a> TxsMptHandler<'a> {
    pub fn new(rpc_url: &'a str, gateway_url: &'a str) -> Result<Self, SnTrieError> {
        let provider = RpcProvider::new(rpc_url, gateway_url);
        Ok(Self {
            provider,
            trie: None,
        })
    }

    pub async fn build_tx_tree_from_block(&mut self, block_number: u64) -> Result<(), SnTrieError> {
        let (txs, expected_commit) = self
            .provider
            .get_block_transactions(block_number)
            .await
            .unwrap();
        let protocol = txs.block_header.starknet_version;
        let tx_final_hashes: Vec<Felt> = txs
            .transactions
            .iter()
            .map(|t| calculate_transaction_hash(t, &protocol))
            .collect();
        self.build_trie(tx_final_hashes, expected_commit)?;
        Ok(())
    }

    pub fn build_trie(
        &mut self,
        txs: Vec<Felt>,
        expected_commit: String,
    ) -> Result<(), SnTrieError> {
        let mut tree: MerkleTree<Pedersen, InMememoryStorage, 64> = Default::default();

        for (idx, hash) in txs.clone().into_iter().enumerate() {
            let idx: Felt = Felt::from(idx as u64);
            tree.set(from_felt_to_bits(&idx), hash).unwrap();
        }

        let (root, root_idx) = tree.commit().unwrap();
        println!("expected root: {:?}", expected_commit);
        println!("root: {:?}", expected_commit);

        // assert_eq!(
        //     root,
        //     Felt::from_bytes_be_slice(hex::decode(expected_commit).unwrap().as_slice())
        // );

        let result_mpt = TxsMpt {
            trie: tree,
            elements: txs,
            root,
            root_idx,
        };

        self.trie = Some(result_mpt);
        Ok(())
    }

    pub fn get_proof(&self, tx_index: u64) -> Result<Vec<TrieNode>, SnTrieError> {
        let idx: Felt = Felt::from(tx_index);
        let root_idx = self.get_root_idx()?;
        let proof = self
            .trie
            .as_ref()
            .ok_or(SnTrieError::TrieNotFound)?
            .trie
            .get_proof(root_idx, from_felt_to_bits(&idx))
            .unwrap()
            .unwrap();

        Ok(proof)
    }

    fn get_root_idx(&self) -> Result<u64, SnTrieError> {
        Ok(self
            .trie
            .as_ref()
            .ok_or(SnTrieError::TrieNotFound)?
            .root_idx)
    }

    pub fn verify_proof(
        &self,
        tx_index: u64,
        proof: Vec<TrieNode>,
    ) -> Result<Membership, SnTrieError> {
        let idx: Felt = Felt::from(tx_index);

        let trie = self.trie.as_ref().ok_or(SnTrieError::TrieNotFound)?;
        let root = trie.root;
        let value = trie
            .elements
            .get(tx_index as usize)
            .ok_or(SnTrieError::InvalidTxIndex)?;

        let result = trie
            .trie
            .verify_proof(root, &from_felt_to_bits(&idx), *value, &proof)
            .unwrap();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATHFINDER_URL: &str = "https://pathfinder.sepolia.iosis.tech/";
    const GATEWAY_URL: &str = "https://alpha-sepolia.starknet.io";

    #[tokio::test]
    async fn test_build_tx_tree_from_block_0() {
        let mut handler = TxsMptHandler::new(PATHFINDER_URL, GATEWAY_URL).unwrap();
        //  # 0.12.3
        let block_number = 7;
        handler
            .build_tx_tree_from_block(block_number)
            .await
            .unwrap();

        let proof = handler.get_proof(1).unwrap();
        let membership: Membership = handler.verify_proof(1, proof).unwrap();

        assert!(membership.is_member());
    }

    #[tokio::test]
    async fn test_build_tx_tree_from_block_1() {
        let mut handler = TxsMptHandler::new(PATHFINDER_URL, GATEWAY_URL).unwrap();
        // # 0.13.0
        let block_number = 35000;
        handler
            .build_tx_tree_from_block(block_number)
            .await
            .unwrap();

        let proof = handler.get_proof(1).unwrap();
        let membership: Membership = handler.verify_proof(1, proof).unwrap();

        assert!(membership.is_member());
    }

    #[tokio::test]
    async fn test_build_tx_tree_from_block_2() {
        let mut handler = TxsMptHandler::new(PATHFINDER_URL, GATEWAY_URL).unwrap();
        // # 0.13.1
        let block_number = 51190;
        handler
            .build_tx_tree_from_block(block_number)
            .await
            .unwrap();

        let proof = handler.get_proof(1).unwrap();
        let membership: Membership = handler.verify_proof(1, proof).unwrap();

        assert!(membership.is_member());
    }

    #[tokio::test]
    async fn test_build_tx_tree_from_block_3() {
        let mut handler = TxsMptHandler::new(PATHFINDER_URL, GATEWAY_URL).unwrap();
        // # 0.13.1.1
        let block_number = 70015;
        handler
            .build_tx_tree_from_block(block_number)
            .await
            .unwrap();

        let proof = handler.get_proof(1).unwrap();
        let membership: Membership = handler.verify_proof(1, proof).unwrap();

        assert!(membership.is_member());
    }

    #[tokio::test]
    async fn test_build_tx_tree_from_block_4() {
        let mut handler = TxsMptHandler::new(PATHFINDER_URL, GATEWAY_URL).unwrap();
        //  # 0.13.2
        let block_number = 99708;
        handler
            .build_tx_tree_from_block(block_number)
            .await
            .unwrap();

        let proof = handler.get_proof(1).unwrap();
        let membership: Membership = handler.verify_proof(1, proof).unwrap();

        assert!(membership.is_member());
    }
}
