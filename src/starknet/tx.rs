use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct BlockWithTxsOutput {
    pub block_hash: String,
    pub block_number: u128,
    pub l1_da_mode: String,
    pub l1_data_gas_price: L1DataGasPrice,
    pub l1_gas_price: L1GasPrice,
    pub new_root: String,
    pub parent_hash: String,
    pub sequencer_address: String,
    pub starknet_version: String,
    pub status: String,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub id: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct Transaction {
    pub calldata: Vec<String>,
    pub max_fee: String,
    pub nonce: String,
    pub sender_address: String,
    pub signature: Vec<String>,
    pub transaction_hash: String,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq, Default)]
pub struct L1DataGasPrice {
    price_in_fri: String,
    price_in_wei: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq, Default)]
pub struct L1GasPrice {
    price_in_fri: String,
    price_in_wei: String,
}
