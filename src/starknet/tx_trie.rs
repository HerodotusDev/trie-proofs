use pathfinder_common::hash::PedersenHash;
use pathfinder_merkle_tree::{tree::MerkleTree, TransactionOrEventTree};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::TxnWithHash;
use url::Url;

use crate::SnTrieError;

use super::{rpc::RpcProvider, tx_hash::calculate_transaction_hash};

pub struct TxsMptHandler {
    provider: RpcProvider,
    trie: Option<TxsMpt>,
}

pub struct TxsMpt {
    pub trie: MerkleTree<PedersenHash, 64>,
    elements: Vec<Felt>,
    commit: Felt,
}

impl TxsMptHandler {
    pub fn new(url: Url) -> Result<Self, SnTrieError> {
        let provider = RpcProvider::new(url);
        Ok(Self {
            provider,
            trie: None,
        })
    }

    pub async fn build_tx_tree_from_block(&mut self, block_number: u64) -> Result<(), SnTrieError> {
        let txs = self
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

        Ok(())
    }

    // pub fn build_trie(&mut self, txs: Vec<Felt>) -> Result<(), SnTrieError> {
    //     let mut tree = MerkleTree::empty();
    //     TransactionOrEventTree

    //     for (idx, hash) in txs.clone().into_iter().enumerate() {
    //         let felt_hash = pathfinder_crypto::Felt::from_be_bytes(hash.to_bytes_be()).unwrap();
    //         let idx: u64 = idx.try_into().unwrap();
    //         let key = idx.to_be_bytes().try_view_bits().to_owned();
    //        tree.set(&NullStorage {}, key, felt_hash);

    //     }

    //     let commit = tree.clone().commit().unwrap();
    //     let converted_commit = Felt::from_bytes_be(&commit.to_be_bytes());

    //     let result_mpt = TxsMpt {
    //         trie: tree,
    //         elements: txs,
    //         commit: converted_commit,
    //     };
    //     Ok(())
    // }
}
