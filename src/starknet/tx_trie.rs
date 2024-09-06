use pathfinder_common::hash::PedersenHash;
use pathfinder_crypto::Felt as PathfinderFelt;
use pathfinder_merkle_tree::{tree::MerkleTree, StatelessStorage, TransactionOrEventTree};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::TxnWithHash;
use url::Url;

use crate::SnTrieError;

use super::{rpc::RpcProvider, tx_hash::calculate_transaction_hash};

pub struct TxsMptHandler<'a> {
    provider: RpcProvider<'a>,
    trie: Option<TxsMpt>,
}

pub struct TxsMpt {
    pub trie: TransactionOrEventTree<PedersenHash>,
    elements: Vec<Felt>,
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
        let mut tree = TransactionOrEventTree::default();

        for (idx, hash) in txs.clone().into_iter().enumerate() {
            let felt_hash = PathfinderFelt::from_be_bytes(hash.to_bytes_be()).unwrap();
            let idx: PathfinderFelt = PathfinderFelt::from_u64(idx as u64);
            tree.set(idx.view_bits().to_owned(), felt_hash).unwrap();
        }

        let (root, root_idx) = tree.commit().unwrap();
        let converted_commit = Felt::from_bytes_be(&root.to_be_bytes());

        assert_eq!(converted_commit.to_string(), expected_commit);

        let result_mpt = TxsMpt {
            trie: tree,
            elements: txs,
            root: converted_commit,
            root_idx,
        };
        Ok(())
    }

    pub fn get_proof(&mut self, tx_index: u64) -> Result<Vec<Vec<u8>>, SnTrieError> {}
}
