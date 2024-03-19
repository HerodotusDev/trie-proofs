use eth_trie::TrieError;
use crate::Error;

pub(crate) mod mpt {
    use std::sync::Arc;
    use alloy_eips::eip2718::Encodable2718;
    use alloy_primitives::{B256, U256};
    use eth_trie::{EthTrie, MemoryDB, Trie};
    use ethereum_types::H256;
    use crate::Error;
    use crate::tx::ConsensusTx;

    pub fn build_tx_tree(txs: Vec<ConsensusTx>, expected_root: B256) -> Result<EthTrie<MemoryDB>, Error> {
        let memdb = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(memdb.clone());

        for (idx, tx) in txs.iter().enumerate() {
            let key = alloy_rlp::encode(&U256::from(idx));
            let rlp = tx.0.encoded_2718();
            trie.insert(key.as_slice(), rlp.as_slice())?;
        }

        if trie.root_hash()?.as_bytes() != expected_root.as_slice() {
            return Err(Error::UnexpectedRoot);
        }

        Ok(trie)
    }

    pub fn get_proof(trie: &mut EthTrie<MemoryDB>, index: u64) -> Result<Vec<Vec<u8>>, Error> {
        let key = alloy_rlp::encode(&U256::from(index));
        let proof = trie.get_proof(key.as_slice())?;

        Ok(proof)
    }

    pub fn verify_proof(trie: &EthTrie<MemoryDB>, tx_root: B256, tx_index: u64, proof: Vec<Vec<u8>>) -> Result<(), Error> {
        match trie.verify_proof(H256::from_slice(tx_root.as_slice()), alloy_rlp::encode(&U256::from(tx_index)).as_slice(), proof) {
            Some(_) => Ok(()),
            None => Err(Error::InvalidMPTProof)
        }
    }

    pub fn build_receipt_tree() {
        unimplemented!()
    }
}

impl From<TrieError> for Error {
    fn from(err: TrieError) -> Self {
        Error::Trie(err)
    }
}