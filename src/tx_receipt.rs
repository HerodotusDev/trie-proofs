use crate::Error;
use alloy_consensus::{Receipt, ReceiptWithBloom};
use alloy_consensus::{ReceiptEnvelope, TxType};
use alloy_eips::eip2718::Decodable2718;
use alloy_network::eip2718::Encodable2718;
use alloy_primitives::U64;
use alloy_primitives::{Bloom, Log, LogData, U8};
use alloy_rpc_types::{Log as RpcLog, TransactionReceipt};

#[derive(Debug, Clone)]
pub struct ConsensusTxReceipt(pub ReceiptEnvelope);

impl ConsensusTxReceipt {
    pub fn rlp_encode(&self) -> Vec<u8> {
        self.0.encoded_2718()
    }

    pub fn rlp_decode(mut data: &[u8]) -> Result<Self, Error> {
        let envelope = ReceiptEnvelope::decode_2718(&mut data).map_err(Error::Rlp)?;
        Ok(ConsensusTxReceipt(envelope))
    }

    pub fn success(&self) -> bool {
        match &self.0 {
            ReceiptEnvelope::Legacy(receipt) => receipt.receipt.success,
            ReceiptEnvelope::Eip2930(receipt) => receipt.receipt.success,
            ReceiptEnvelope::Eip1559(receipt) => receipt.receipt.success,
            ReceiptEnvelope::Eip4844(receipt) => receipt.receipt.success,
        }
    }

    pub fn cumulative_gas_used(&self) -> u64 {
        match &self.0 {
            ReceiptEnvelope::Legacy(receipt) => receipt.receipt.cumulative_gas_used,
            ReceiptEnvelope::Eip2930(receipt) => receipt.receipt.cumulative_gas_used,
            ReceiptEnvelope::Eip1559(receipt) => receipt.receipt.cumulative_gas_used,
            ReceiptEnvelope::Eip4844(receipt) => receipt.receipt.cumulative_gas_used,
        }
    }

    pub fn logs(&self) -> Vec<Log<LogData>> {
        match &self.0 {
            ReceiptEnvelope::Legacy(receipt) => receipt.receipt.logs.clone(),
            ReceiptEnvelope::Eip2930(receipt) => receipt.receipt.logs.clone(),
            ReceiptEnvelope::Eip1559(receipt) => receipt.receipt.logs.clone(),
            ReceiptEnvelope::Eip4844(receipt) => receipt.receipt.logs.clone(),
        }
    }

    pub fn bloom(&self) -> Bloom {
        match &self.0 {
            ReceiptEnvelope::Legacy(receipt) => receipt.bloom,
            ReceiptEnvelope::Eip2930(receipt) => receipt.bloom,
            ReceiptEnvelope::Eip1559(receipt) => receipt.bloom,
            ReceiptEnvelope::Eip4844(receipt) => receipt.bloom,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RpcTxReceipt(pub TransactionReceipt);

impl TryFrom<RpcTxReceipt> for ConsensusTxReceipt {
    type Error = Error;
    fn try_from(tx: RpcTxReceipt) -> Result<ConsensusTxReceipt, Error> {
        match &tx.version()? {
            TxType::Legacy => {
                let res = ReceiptEnvelope::Legacy(ReceiptWithBloom {
                    receipt: Receipt {
                        success: tx.success(),
                        cumulative_gas_used: tx.cumulative_gas_used(),
                        logs: tx.logs(),
                    },
                    bloom: tx.bloom(),
                });
                Ok(ConsensusTxReceipt(res))
            }
            TxType::Eip2930 => {
                let res = ReceiptEnvelope::Eip2930(ReceiptWithBloom {
                    receipt: Receipt {
                        success: tx.success(),
                        cumulative_gas_used: tx.cumulative_gas_used(),
                        logs: tx.logs(),
                    },
                    bloom: tx.bloom(),
                });
                Ok(ConsensusTxReceipt(res))
            }
            TxType::Eip1559 => {
                let res = ReceiptEnvelope::Eip1559(ReceiptWithBloom {
                    receipt: Receipt {
                        success: tx.success(),
                        cumulative_gas_used: tx.cumulative_gas_used(),
                        logs: tx.logs(),
                    },
                    bloom: tx.bloom(),
                });
                Ok(ConsensusTxReceipt(res))
            }
            TxType::Eip4844 => {
                let res = ReceiptEnvelope::Eip4844(ReceiptWithBloom {
                    receipt: Receipt {
                        success: tx.success(),
                        cumulative_gas_used: tx.cumulative_gas_used(),
                        logs: tx.logs(),
                    },
                    bloom: tx.bloom(),
                });
                Ok(ConsensusTxReceipt(res))
            }
        }
    }
}

impl RpcTxReceipt {
    fn version(&self) -> Result<TxType, Error> {
        match &self.0.transaction_type {
            // Legacy
            tx_type if tx_type == &U8::from(0) => Ok(TxType::Legacy),
            // EIP-2930
            tx_type if tx_type == &U8::from(1) => Ok(TxType::Eip2930),
            // EIP-1559
            tx_type if tx_type == &U8::from(2) => Ok(TxType::Eip1559),
            // EIP-4844
            tx_type if tx_type == &U8::from(3) => Ok(TxType::Eip4844),
            _ => Err(Error::InvalidTxVersion),
        }
    }

    fn success(&self) -> bool {
        match &self.0.status_code {
            Some(status) => status == &U64::from(1),
            None => false,
        }
    }

    fn cumulative_gas_used(&self) -> u64 {
        self.0.cumulative_gas_used.try_into().unwrap()
    }

    fn logs(&self) -> Vec<alloy_primitives::Log<LogData>> {
        let mut logs = Vec::new();
        for log in &self.0.logs {
            let rpc_log: RpcLog = log.clone();
            let log_data = LogData::try_from(rpc_log).unwrap();
            let result = Log {
                address: log.address,
                data: log_data,
            };
            logs.push(result);
        }

        logs
    }

    fn bloom(&self) -> Bloom {
        self.0.logs_bloom
    }
}
