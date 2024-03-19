mod proof;
mod rpc;
mod tx;

use crate::tx::{ConsensusTx, RpcTx};

use alloy_primitives::{hex, B256};

use alloy_transport::{RpcError, TransportErrorKind};
use eth_trie::TrieError;
use proof::mpt;
use rpc::RpcProvider;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let provider = RpcProvider::new();
    let tx_hash = B256::from(hex!(
        "ef1503cc8bd82da1552389183a097126bae21a889390a7be556b1f69d8c75c29"
    ));
    let height = RpcProvider::get_tx_block_height(&provider, tx_hash).await?;
    let (txs, tx_root, tx_index) = provider.get_block_transactions(height, tx_hash).await?;
    let converted: Vec<ConsensusTx> = txs
        .iter()
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
    InvalidMPTProof,
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
    Signature,
}

// #[cfg(test)]
// mod tests {
// 19462159 -> all tx types except 4844
// 19460281 -> 4844

// }
