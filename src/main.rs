mod rpc;
mod tx;

use rpc::RpcProvider;
use alloy_primitives::{U256, B256};
use alloy_rlp;
use alloy_provider::{Provider};
use alloy_consensus::SignableTransaction;
use alloy_rlp::Encodable;
use alloy_network::eip2718::Encodable2718;
use std::sync::Arc;
use eth_trie::MemoryDB;
use eth_trie::{EthTrie, Trie, TrieError};
use crate::tx::{ConsensusTx, RpcTx};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let provider = RpcProvider::new();
    let height = RpcProvider::get_tx_block_height(&provider, B256::from([38,86,13,104,96,219,186,48,160,248,209,237,235,63,142,200,50,170,203,16,24,3,139,33,33,179,162,46,93,52,59,44])).await?;
    println!("Height: {:?}", height);
    let txs = provider.get_block_transactions(height).await?;
    println!("Tx count: {:?}", txs.len());
     // Refactored line: Properly handling errors within the map operation before collecting.
    let converted: Vec<ConsensusTx> = txs.iter()
        .map(|tx| RpcTx(tx.clone()).try_into())
        .collect::<Result<Vec<_>, _>>().map_err(|_| Error::ConversionError)?;

    let memdb = Arc::new(MemoryDB::new(true));
    let mut trie = EthTrie::new(memdb.clone());

    converted.iter().enumerate().for_each(|(idx, tx)| {
        let key = alloy_rlp::encode(&U256::from(idx));
        let rlp = tx.0.encoded_2718();
        trie.insert(key.as_slice(), rlp.as_slice()).unwrap();
    });

    let new_root = trie.root_hash();
    println!("NeRoot: {:?}", new_root);
    Ok(())

}


#[derive(Debug)]
pub enum Error {
    Trie(TrieError),
    Rlp(alloy_rlp::Error),
    RPC(String),
    TxNotFound,
    InvalidTxVersion,
    ConversionError
}