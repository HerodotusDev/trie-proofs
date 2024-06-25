use alloy::transports::{RpcError, TransportErrorKind};
use eth_trie::TrieError;

mod rpc;
pub mod tx;
pub mod tx_receipt;
pub mod tx_receipt_trie;
pub mod tx_trie;

#[derive(Debug)]
pub enum Error {
    Trie(TrieError),
    Eip(alloy::eips::eip2718::Eip2718Error),
    Rlp(alloy_rlp::Error),
    RPC(RpcError<TransportErrorKind>),
    TxNotFound,
    BlockNotFound,
    InvalidTxVersion,
    ConversionError(Field),
    UnexpectedRoot,
    InvalidMPTProof,
    TrieNotFound,
    FieldNotFound,
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

impl From<TrieError> for Error {
    fn from(err: TrieError) -> Self {
        Error::Trie(err)
    }
}
