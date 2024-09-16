use thiserror::Error;

#[derive(Error, Debug)]
pub enum SnTrieError {
    #[error("Starknet error: {0}")]
    GatewayError(u16),
    #[error("Trie not found")]
    TrieNotFound,

    #[error("Invalid transaction index")]
    InvalidTxIndex,

    #[error("Invalid commit")]
    InvalidCommitment,

    #[error("Verification error")]
    VerificationError,

    #[error("Unsupported protocol")]
    UnsupportedProtocol,
}
