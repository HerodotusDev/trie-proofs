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

        assert_eq!(root.to_string(), expected_commit);

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
        // let root_idx = self.get_root_idx()?;
        let trie = self.trie.as_ref().ok_or(SnTrieError::TrieNotFound)?;
        let root = trie.root;

        let result = trie
            .trie
            .verify_proof(root, &from_felt_to_bits(&idx), idx, &proof)
            .unwrap();
        Ok(result)
    }
}
