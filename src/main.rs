mod rpc;
mod tx;
mod proof;

use proof::mpt;
use rpc::RpcProvider;
use alloy_primitives::{B256, hex};
use alloy_rlp;
use alloy_provider::{Provider};
use alloy_consensus::SignableTransaction;
use alloy_rlp::Encodable;
use alloy_network::eip2718::Encodable2718;
use alloy_transport::{RpcError, TransportErrorKind};
use eth_trie::{TrieError};
use crate::tx::{ConsensusTx, RpcTx};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let provider = RpcProvider::new();
    let tx_hash = B256::from(hex!("ef1503cc8bd82da1552389183a097126bae21a889390a7be556b1f69d8c75c29"));
    let height = RpcProvider::get_tx_block_height(&provider, tx_hash.clone()).await?;
    let (txs, tx_root, tx_index) = provider.get_block_transactions(height, tx_hash).await?;
    let converted: Vec<ConsensusTx> = txs.iter()
        .map(|tx| RpcTx(tx.clone()).try_into())
        .collect::<Result<Vec<_>, _>>()?;

    let mut trie = mpt::build_tx_tree(converted, tx_root)?;
    let proof = mpt::get_proof(&mut trie, tx_index)?;
    mpt::verify_proof(&trie, tx_root, tx_index, proof)?;

    Ok(())
}


#[derive(Debug)]
pub enum Error {
    Trie(TrieError),
    Rlp(alloy_rlp::Error),
    RPC(RpcError<TransportErrorKind>),
    TxNotFound,
    BlockNotFound,
    InvalidTxVersion,
    ConversionError(Field),
    UnexpectedRoot,
    InvalidMPTProof
}
#[derive(Debug)]
pub enum Field {
    ChainId,
    Nonce,
    GasPrice,
    GasLimit,
    Input,
    AccessList,
    MaxFeePerGas,
    MaxPriorityFeePerGas,
    MaxFeePerBlobGas,
    Signature
}

// #[cfg(test)]
// mod tests {
    // 19462159 -> all tx types except 4844
    // 19460281 -> 4844

// }