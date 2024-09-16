use starknet_types_core::hash::{Pedersen, Poseidon};
use starknet_types_core::{felt::Felt, hash::StarkHash};
use starknet_types_rpc::{Txn, TxnWithHash};

pub fn calculate_transaction_hash(tx: &TxnWithHash<Felt>, protocol_version: &str) -> Felt {
    if protocol_version >= "0.13.2" {
        let mut hash_chains = vec![tx.transaction_hash];

        match &tx.transaction {
            Txn::Deploy(_) | Txn::L1Handler(_) => {
                hash_chains.push(Felt::ZERO);
            }
            tx_type => {
                let signatures = match &tx_type {
                    Txn::Invoke(invoke_tx) => match invoke_tx {
                        starknet_types_rpc::InvokeTxn::V0(tx) => &tx.signature,
                        starknet_types_rpc::InvokeTxn::V1(tx) => &tx.signature,
                        starknet_types_rpc::InvokeTxn::V3(tx) => &tx.signature,
                    },
                    Txn::Declare(declare_tx) => match declare_tx {
                        starknet_types_rpc::DeclareTxn::V0(tx) => &tx.signature,
                        starknet_types_rpc::DeclareTxn::V1(tx) => &tx.signature,
                        starknet_types_rpc::DeclareTxn::V2(tx) => &tx.signature,
                        starknet_types_rpc::DeclareTxn::V3(tx) => &tx.signature,
                    },
                    Txn::DeployAccount(deploy_account_tx) => match deploy_account_tx {
                        starknet_types_rpc::DeployAccountTxn::V1(tx) => &tx.signature,
                        starknet_types_rpc::DeployAccountTxn::V3(tx) => &tx.signature,
                    },
                    _ => unreachable!(),
                };
                for sig in signatures {
                    hash_chains.push(*sig);
                }
            }
        }

        Poseidon::hash_array(&hash_chains)
    } else if protocol_version < "0.11.1" {
        let (transaction_hash, signature_hash) = match &tx.transaction {
            Txn::Invoke(invoke_tx) => {
                let signatures = match invoke_tx {
                    starknet_types_rpc::InvokeTxn::V0(tx) => &tx.signature,
                    starknet_types_rpc::InvokeTxn::V1(tx) => &tx.signature,
                    starknet_types_rpc::InvokeTxn::V3(tx) => &tx.signature,
                };
                (tx.transaction_hash, Pedersen::hash_array(signatures))
            }
            _ => (tx.transaction_hash, Pedersen::hash_array(&[])),
        };

        Pedersen::hash(&transaction_hash, &signature_hash)
    } else if protocol_version < "0.13.2" {
        let (transaction_hash, signature_hash) = match &tx.transaction {
            Txn::Deploy(_) | Txn::L1Handler(_) => (tx.transaction_hash, Pedersen::hash_array(&[])),
            _ => {
                let signatures = match &tx.transaction {
                    Txn::Invoke(invoke_tx) => match invoke_tx {
                        starknet_types_rpc::InvokeTxn::V0(tx) => &tx.signature,
                        starknet_types_rpc::InvokeTxn::V1(tx) => &tx.signature,
                        starknet_types_rpc::InvokeTxn::V3(tx) => &tx.signature,
                    },
                    Txn::Declare(declare_tx) => match declare_tx {
                        starknet_types_rpc::DeclareTxn::V0(tx) => &tx.signature,
                        starknet_types_rpc::DeclareTxn::V1(tx) => &tx.signature,
                        starknet_types_rpc::DeclareTxn::V2(tx) => &tx.signature,
                        starknet_types_rpc::DeclareTxn::V3(tx) => &tx.signature,
                    },
                    Txn::DeployAccount(deploy_account_tx) => match deploy_account_tx {
                        starknet_types_rpc::DeployAccountTxn::V1(tx) => &tx.signature,
                        starknet_types_rpc::DeployAccountTxn::V3(tx) => &tx.signature,
                    },
                    _ => unreachable!(),
                };
                (tx.transaction_hash, Pedersen::hash_array(signatures))
            }
        };

        Pedersen::hash(&transaction_hash, &signature_hash)
    } else {
        panic!("Invalid protocol version")
    }
}
