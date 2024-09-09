use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct StarknetBlock {
    pub block_hash: String,
    pub parent_block_hash: String,
    pub block_number: u64,
    pub state_root: String,
    pub transaction_commitment: String,
    pub event_commitment: String,
    pub receipt_commitment: Option<String>,
    pub state_diff_commitment: Option<String>,
    pub state_diff_length: Option<u32>,
    pub status: String,
    pub l1_da_mode: String,
    pub l1_gas_price: GasPrice,
    pub l1_data_gas_price: GasPrice,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
    pub sequencer_address: String,
    pub transaction_receipts: Vec<TransactionReceipt>,
    pub starknet_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasPrice {
    pub price_in_wei: String,
    pub price_in_fri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_hash: String,
    pub version: String,
    pub max_fee: Option<String>,
    pub signature: Vec<String>,
    pub nonce: String,
    pub sender_address: Option<String>,
    pub calldata: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub transaction_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub execution_status: String,
    pub transaction_index: u32,
    pub transaction_hash: String,
    pub l2_to_l1_messages: Vec<String>,
    pub events: Vec<Event>,
    pub execution_resources: ExecutionResources,
    pub actual_fee: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub from_address: String,
    pub keys: Vec<String>,
    pub data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResources {
    pub n_steps: u32,
    pub builtin_instance_counter: HashMap<String, u32>,
    pub n_memory_holes: u32,
    pub data_availability: Option<DataAvailability>,
    pub total_gas_consumed: Option<GasConsumed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataAvailability {
    pub l1_gas: u32,
    pub l1_data_gas: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasConsumed {
    pub l1_gas: u32,
    pub l1_data_gas: u32,
}
