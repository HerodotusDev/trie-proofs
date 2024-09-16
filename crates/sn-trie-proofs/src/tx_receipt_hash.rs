use starknet_types_core::{
    felt::Felt,
    hash::{Poseidon, StarkHash},
};
use starknet_types_rpc::{MsgToL1, TransactionAndReceipt};

pub fn calculate_receipt_hash(receipt: &TransactionAndReceipt<Felt>, l1_gas: u64) -> Felt {
    let mut hash_chains = vec![];
    let common_properties = match &receipt.receipt {
        starknet_types_rpc::TxnReceipt::Declare(tx_receipt) => {
            &tx_receipt.common_receipt_properties
        }
        starknet_types_rpc::TxnReceipt::Deploy(tx_receipt) => &tx_receipt.common_receipt_properties,
        starknet_types_rpc::TxnReceipt::DeployAccount(tx_receipt) => {
            &tx_receipt.common_receipt_properties
        }
        starknet_types_rpc::TxnReceipt::Invoke(tx_receipt) => &tx_receipt.common_receipt_properties,
        starknet_types_rpc::TxnReceipt::L1Handler(tx_receipt) => {
            &tx_receipt.common_receipt_properties
        }
    };

    hash_chains.push(common_properties.transaction_hash);
    hash_chains.push(common_properties.actual_fee.amount);
    hash_chains.push(calculate_messages_sent_hash(
        &common_properties.messages_sent,
    ));
    // TODO: calculate_revert_reason_hash
    hash_chains.push(Felt::ZERO);

    // chain_gas_consumed
    hash_chains.push(Felt::ZERO);
    hash_chains.push(l1_gas.into());
    hash_chains.push(
        common_properties
            .execution_resources
            .data_availability
            .l1_data_gas
            .into(),
    );

    Poseidon::hash_array(&hash_chains)
}

pub fn calculate_messages_sent_hash(messages: &Vec<MsgToL1<Felt>>) -> Felt {
    let mut hash_chains = vec![];
    hash_chains.push(Felt::from(messages.len() as u64));
    for message in messages {
        hash_chains.push(message.from_address);
        hash_chains.push(message.to_address);
        hash_chains.push(Felt::from(message.payload.len()));
        for p in message.payload.clone() {
            hash_chains.push(p);
        }
    }
    Poseidon::hash_array(&hash_chains)
}

// Returns starknet-keccak of the revert reason ASCII string, or 0 if the transaction succeeded.
// pub fn calculate_revert_reason_hash(execution_status: String, revert_reason: String) -> Felt {

// }
