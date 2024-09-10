use crate::evm::error::EthTrieError;
use alloy::consensus::{Eip658Value, Receipt, ReceiptWithBloom, TxReceipt};
use alloy::consensus::{ReceiptEnvelope, TxType};
use alloy::eips::eip2718::Decodable2718;
use alloy::network::eip2718::Encodable2718;

use alloy::primitives::{Bloom, Log, LogData};
use alloy::rpc::types::{Log as RpcLog, TransactionReceipt};

#[derive(Debug, Clone)]
pub struct ConsensusTxReceipt(pub ReceiptEnvelope);

impl ConsensusTxReceipt {
    pub fn rlp_encode(&self) -> Vec<u8> {
        self.0.encoded_2718()
    }

    pub fn rlp_decode(mut data: &[u8]) -> Result<Self, EthTrieError> {
        let envelope = ReceiptEnvelope::decode_2718(&mut data).map_err(EthTrieError::Eip)?;
        Ok(ConsensusTxReceipt(envelope))
    }

    pub fn status(&self) -> &Eip658Value {
        match &self.0 {
            ReceiptEnvelope::Legacy(receipt) => receipt.receipt.status_or_post_state(),
            ReceiptEnvelope::Eip2930(receipt) => receipt.receipt.status_or_post_state(),
            ReceiptEnvelope::Eip1559(receipt) => receipt.receipt.status_or_post_state(),
            ReceiptEnvelope::Eip4844(receipt) => receipt.receipt.status_or_post_state(),
            _ => todo!(),
        }
    }

    pub fn cumulative_gas_used(&self) -> u128 {
        match &self.0 {
            ReceiptEnvelope::Legacy(receipt) => receipt.receipt.cumulative_gas_used,
            ReceiptEnvelope::Eip2930(receipt) => receipt.receipt.cumulative_gas_used,
            ReceiptEnvelope::Eip1559(receipt) => receipt.receipt.cumulative_gas_used,
            ReceiptEnvelope::Eip4844(receipt) => receipt.receipt.cumulative_gas_used,
            _ => todo!(),
        }
    }

    pub fn logs(&self) -> Vec<Log<LogData>> {
        match &self.0 {
            ReceiptEnvelope::Legacy(receipt) => receipt.receipt.logs.clone(),
            ReceiptEnvelope::Eip2930(receipt) => receipt.receipt.logs.clone(),
            ReceiptEnvelope::Eip1559(receipt) => receipt.receipt.logs.clone(),
            ReceiptEnvelope::Eip4844(receipt) => receipt.receipt.logs.clone(),
            _ => todo!(),
        }
    }

    pub fn bloom(&self) -> Bloom {
        match &self.0 {
            ReceiptEnvelope::Legacy(receipt) => receipt.bloom(),
            ReceiptEnvelope::Eip2930(receipt) => receipt.bloom(),
            ReceiptEnvelope::Eip1559(receipt) => receipt.bloom(),
            ReceiptEnvelope::Eip4844(receipt) => receipt.bloom(),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RpcTxReceipt(pub TransactionReceipt);

impl TryFrom<RpcTxReceipt> for ConsensusTxReceipt {
    type Error = EthTrieError;
    fn try_from(tx: RpcTxReceipt) -> Result<ConsensusTxReceipt, EthTrieError> {
        match &tx.version()? {
            TxType::Legacy => {
                let res = ReceiptEnvelope::Legacy(ReceiptWithBloom {
                    receipt: Receipt {
                        status: Eip658Value::from(tx.status()),
                        cumulative_gas_used: tx.cumulative_gas_used(),
                        logs: tx.logs(),
                    },
                    logs_bloom: tx.bloom(),
                });
                Ok(ConsensusTxReceipt(res))
            }
            TxType::Eip2930 => {
                let res = ReceiptEnvelope::Eip2930(ReceiptWithBloom {
                    receipt: Receipt {
                        status: Eip658Value::from(tx.status()),
                        cumulative_gas_used: tx.cumulative_gas_used(),
                        logs: tx.logs(),
                    },
                    logs_bloom: tx.bloom(),
                });
                Ok(ConsensusTxReceipt(res))
            }
            TxType::Eip1559 => {
                let res = ReceiptEnvelope::Eip1559(ReceiptWithBloom {
                    receipt: Receipt {
                        status: Eip658Value::from(tx.status()),
                        cumulative_gas_used: tx.cumulative_gas_used(),
                        logs: tx.logs(),
                    },
                    logs_bloom: tx.bloom(),
                });
                Ok(ConsensusTxReceipt(res))
            }
            TxType::Eip4844 => {
                let res = ReceiptEnvelope::Eip4844(ReceiptWithBloom {
                    receipt: Receipt {
                        status: Eip658Value::from(tx.status()),
                        cumulative_gas_used: tx.cumulative_gas_used(),
                        logs: tx.logs(),
                    },
                    logs_bloom: tx.bloom(),
                });
                Ok(ConsensusTxReceipt(res))
            }
        }
    }
}

impl RpcTxReceipt {
    fn version(&self) -> Result<TxType, EthTrieError> {
        Ok(self.0.transaction_type())
    }

    fn status(&self) -> bool {
        self.0.status()
    }

    fn cumulative_gas_used(&self) -> u128 {
        self.0.inner.cumulative_gas_used()
    }

    fn logs(&self) -> Vec<alloy::primitives::Log<LogData>> {
        let mut logs = Vec::new();
        for log in self.0.inner.logs() {
            let rpc_log: RpcLog = log.clone();
            let result = rpc_log.inner;
            logs.push(result);
        }
        logs
    }

    fn bloom(&self) -> Bloom {
        self.0.inner.bloom()
    }
}
