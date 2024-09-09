use core::fmt;

use alloy::transports::{RpcError, TransportErrorKind};
use eth_trie::TrieError;
use thiserror::Error;

mod rpc;
pub mod starknet;
pub mod tx;
pub mod tx_receipt;
pub mod tx_receipt_trie;
pub mod tx_trie;

#[derive(Error, Debug)]
pub enum SnTrieError {
    #[error("Starknet error: {0}")]
    GatewayError(u16),
    #[error("Trie not found")]
    TrieNotFound,

    #[error("Invalid transaction index")]
    InvalidTxIndex,
}

#[derive(Error, Debug)]
pub enum EthTrieError {
    #[error("Trie error: {0}")]
    Trie(TrieError),
    #[error("EIP error: {0}")]
    Eip(alloy::eips::eip2718::Eip2718Error),
    #[error("RLP error: {0}")]
    Rlp(alloy_rlp::Error),
    #[error("RPC error: {0}")]
    RPC(RpcError<TransportErrorKind>),
    #[error("Transaction not found")]
    TxNotFound,
    #[error("Block not found")]
    BlockNotFound,
    #[error("Invalid transaction version")]
    InvalidTxVersion,
    #[error("Error converting field: {0}")]
    ConversionError(Field),
    #[error("Unexpected root")]
    UnexpectedRoot,
    #[error("Invalid mpt proof")]
    InvalidMPTProof,
    #[error("Invalid transaction trie")]
    TrieNotFound,
    #[error("Field not found")]
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

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Field::ChainId => write!(f, "chain_id"),
            Field::Nonce => write!(f, "nonce"),
            Field::GasPrice => write!(f, "gas_price"),
            Field::GasLimit => write!(f, "gas_limit"),
            Field::Input => write!(f, "input"),
            Field::AccessList => write!(f, "access_list"),
            Field::MaxFeePerGas => write!(f, "max_fee_per_gas"),
            Field::MaxPriorityFeePerGas => write!(f, "max_priority_fee_per_gas"),
            Field::MaxFeePerBlobGas => write!(f, "max_fee_per_blob_gas"),
            Field::Signature => write!(f, "signature"),
        }
    }
}

impl From<TrieError> for EthTrieError {
    fn from(err: TrieError) -> Self {
        EthTrieError::Trie(err)
    }
}
