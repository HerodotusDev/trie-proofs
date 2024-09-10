use crate::SnTrieError;
use sn_trie::conversion::from_u64_to_bits;
use sn_trie::{node::TrieNode, storage::memory::InMememoryStorage};
use sn_trie::{Membership, MerkleTree};
use starknet_types_core::{felt::Felt, hash::Pedersen};

use super::rpc::GATEWAY_URL;
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
    pub fn new(rpc_url: &'a str) -> Result<Self, SnTrieError> {
        let provider = RpcProvider::new(rpc_url, GATEWAY_URL);
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

        self.build_trie(tx_final_hashes, &expected_commit)?;
        Ok(())
    }

    pub fn build_trie(&mut self, txs: Vec<Felt>, expected_commit: &str) -> Result<(), SnTrieError> {
        let mut tree: MerkleTree<Pedersen, InMememoryStorage, 64> = Default::default();

        for (idx, hash) in txs.clone().into_iter().enumerate() {
            let idx: u64 = idx.try_into().unwrap();
            let key = from_u64_to_bits(idx);
            tree.set(key, hash).unwrap();
        }

        let (root, root_idx) = tree.commit().unwrap();

        let cleaned_expected_commit = expected_commit.trim_matches('"').to_string();
        assert_eq!(cleaned_expected_commit, root.to_hex_string());
        if cleaned_expected_commit != root.to_hex_string() {
            return Err(SnTrieError::InvalidCommitment);
        }

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
        let root_idx = self.get_root_idx()?;
        let proof = self
            .trie
            .as_ref()
            .ok_or(SnTrieError::TrieNotFound)?
            .trie
            .get_proof(root_idx, from_u64_to_bits(tx_index))
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
        let trie = self.trie.as_ref().ok_or(SnTrieError::TrieNotFound)?;
        let root = trie.root;
        let value = trie
            .elements
            .get(tx_index as usize)
            .ok_or(SnTrieError::InvalidTxIndex)?;

        let result = trie
            .trie
            .verify_proof(root, &from_u64_to_bits(tx_index), *value, &proof)
            .unwrap();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATHFINDER_URL: &str = "https://pathfinder.sepolia.iosis.tech/";

    #[tokio::test]
    async fn test_build_tree() {
        let tx_final = [
            Felt::from_hex_unchecked(
                "0x7ddafca73b3f2c36ca920b34f3536fdbe876511b9d582a1127fa0780977c257",
            ),
            Felt::from_hex_unchecked(
                "0x4ecbdbef65a58c90abf2d03ae9dc6c66ad4331bccd098a16f8dd6a0976152f4",
            ),
            Felt::from_hex_unchecked(
                "0x53222976c5360d0b9f0380f4a8b2a0f135540eb64eb021113282b4d519df2cf",
            ),
            Felt::from_hex_unchecked(
                "0x39c8f931e51038872c7a2650a68c388e9b6abd268ea710a950f845119e2f0bb",
            ),
            Felt::from_hex_unchecked(
                "0x696713c5328e75e97f2fa60fc6296c84def311b1bace7e19571efc58801efc0",
            ),
            Felt::from_hex_unchecked(
                "0x7553ae1155d78c96ca1bf14ff36e283ae56e1d23c4c64c61c250b5c67a3c564",
            ),
            Felt::from_hex_unchecked(
                "0x5f26a24b17932cc85462d1e6272d9e7bce6db27640884e64582253580a57a1a",
            ),
            Felt::from_hex_unchecked(
                "0xfb1f8879a89f5fed0cfa2848cf77ad940d1c9e2a5a8dd3b41075c96dee3141",
            ),
            Felt::from_hex_unchecked(
                "0x4ba17bc1bde8b5412ad0c3bad9ee89d9a00e373568b9097a289f045cad83b41",
            ),
            Felt::from_hex_unchecked(
                "0x506cd52babd14d4386c50053324777fb6245f43da25162bb99decef0e9a2ec7",
            ),
            Felt::from_hex_unchecked(
                "0x6f8f0dfb7dfc1f9a4ff37680cfc41d0fdc3f57bfcd0d1a88310cf44b6675ed3",
            ),
            Felt::from_hex_unchecked(
                "0x3d7e6dbb27dad9327640e4ae7e373186e3c01e05a24d621eb106845a68a1e",
            ),
            Felt::from_hex_unchecked(
                "0x62d36d3c9cc8e499276632bb9d3d428d28665243f722f3d5071c48b2c1ff3a1",
            ),
            Felt::from_hex_unchecked(
                "0x6c7b8f6526d8d345a39bae9d78f73168491eedb2d9c46430c831d0705e1b777",
            ),
            Felt::from_hex_unchecked(
                "0x79a1896bd14b45593d6d788e9c43f8fb5a8e66c07dccf85cbbb21b4b03b44da",
            ),
            Felt::from_hex_unchecked(
                "0x187d9280d359b6babad05ae9ae1549231c0eb8ef084ea7500b04c74aa7bf945",
            ),
            Felt::from_hex_unchecked(
                "0x4c9792370f39c29ca3643c80a8903b31db675cfcc86970c868652e7dd6f139b",
            ),
        ];
        let mut handler = TxsMptHandler::new(PATHFINDER_URL).unwrap();
        handler
            .build_trie(
                tx_final.to_vec(),
                "0x5b209de02dadbe60f29809c4409541b3b1c8cac7260005e1ecad82bf8a9c524",
            )
            .unwrap();
    }

    #[tokio::test]
    async fn test_build_tx_tree_from_block_0() {
        let mut handler = TxsMptHandler::new(PATHFINDER_URL).unwrap();
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
        let mut handler = TxsMptHandler::new(PATHFINDER_URL).unwrap();
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
        let mut handler = TxsMptHandler::new(PATHFINDER_URL).unwrap();
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
        let mut handler = TxsMptHandler::new(PATHFINDER_URL).unwrap();
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
        let mut handler = TxsMptHandler::new(PATHFINDER_URL).unwrap();
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
