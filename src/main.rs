mod rpc;
mod tx;

use crate::tx::{ConsensusTx, RpcTx};
use alloy_network::eip2718::Encodable2718;
use alloy_primitives::{B256, U256, U64};
use clap::Parser;
use eth_trie::MemoryDB;
use eth_trie::{EthTrie, Trie, TrieError};
use rpc::RpcProvider;
use std::str::FromStr;
use std::sync::Arc;

/// Simple Herodotus Data Processor CLI to handle tasks and datalakes
#[derive(Debug, Parser)]
#[command(name = "hdp")]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    tx_hash: String,

    #[arg(default_value_t = 1, short, long)]
    chain_id: u64,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let tx_hash = B256::from_str(&args.tx_hash).map_err(|_| Error::ConversionError)?;
    let chain_id = U64::from(args.chain_id);
    let provider = RpcProvider::new();
    let height = RpcProvider::get_tx_block_height(&provider, tx_hash).await?;
    println!("Height: {:?}", height);
    let mut txs_with_root = provider.get_block_transactions(height).await?;
    println!("Tx count: {:?}", txs_with_root.transactions.len());
    println!("Root from header: {:?}", txs_with_root.root);
    // Refactored line: Properly handling errors within the map operation before collecting.
    let converted: Vec<ConsensusTx> = txs_with_root
        .transactions
        .iter_mut()
        .map(|tx| {
            if tx.chain_id.is_none() {
                tx.chain_id = Some(chain_id);
            }
            RpcTx(tx.clone()).try_into()
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Error::ConversionError)?;

    let memdb = Arc::new(MemoryDB::new(true));
    let mut trie = EthTrie::new(memdb.clone());

    converted.iter().enumerate().for_each(|(idx, tx)| {
        let key = alloy_rlp::encode(U256::from(idx));
        let rlp = tx.0.encoded_2718();
        trie.insert(key.as_slice(), rlp.as_slice()).unwrap();
    });

    let new_root = trie.root_hash().unwrap();
    println!("NeRoot: {:?}", new_root);

    if txs_with_root.root != new_root.to_fixed_bytes() {
        // TODO: If eip4844 tx exist, currently root is not matching
        println!("Root mismatch");
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Trie(TrieError),
    Rlp(alloy_rlp::Error),
    RPC(String),
    TxNotFound,
    InvalidTxVersion,
    ConversionError,
}
