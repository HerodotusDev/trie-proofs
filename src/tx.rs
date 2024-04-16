use crate::{Error, Field};
use alloy_consensus::{
    SignableTransaction, TxEip1559, TxEip2930, TxEip4844, TxEnvelope, TxLegacy, TxType,
};
use alloy_consensus::{Transaction as ConsensusTransaction, TxEip4844Variant};
use alloy_eips::eip2718::Decodable2718;
use alloy_eips::eip2930::AccessList;
use alloy_eips::eip2930::AccessListItem;
use alloy_network::eip2718::Encodable2718;
use alloy_primitives::{ChainId, FixedBytes, Parity, Signature, TxKind, U256, U64};
use alloy_rpc_types::Transaction;

#[derive(Debug, Clone)]
pub struct ConsensusTx(pub TxEnvelope);

impl ConsensusTx {
    pub fn rlp_encode(&self) -> Vec<u8> {
        self.0.encoded_2718()
    }

    pub fn rlp_decode(mut data: &[u8]) -> Result<Self, Error> {
        let tx = TxEnvelope::decode_2718(&mut data).map_err(Error::Rlp)?;
        Ok(ConsensusTx(tx))
    }

    pub fn nonce(&self) -> u64 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().nonce(),
            TxEnvelope::Eip2930(tx) => tx.tx().nonce(),
            TxEnvelope::Eip1559(tx) => tx.tx().nonce(),
            TxEnvelope::Eip4844(tx) => tx.tx().nonce(),
        }
    }

    pub fn gas_limit(&self) -> u64 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().gas_limit(),
            TxEnvelope::Eip2930(tx) => tx.tx().gas_limit(),
            TxEnvelope::Eip1559(tx) => tx.tx().gas_limit(),
            TxEnvelope::Eip4844(tx) => tx.tx().gas_limit(),
        }
    }

    pub fn gas_price(&self) -> Option<U256> {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().gas_price(),
            TxEnvelope::Eip2930(tx) => tx.tx().gas_price(),
            TxEnvelope::Eip1559(tx) => tx.tx().gas_price(),
            TxEnvelope::Eip4844(tx) => tx.tx().gas_price(),
        }
    }

    pub fn to(&self) -> TxKind {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().to(),
            TxEnvelope::Eip2930(tx) => tx.tx().to(),
            TxEnvelope::Eip1559(tx) => tx.tx().to(),
            TxEnvelope::Eip4844(tx) => tx.tx().to(),
        }
    }

    pub fn value(&self) -> U256 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().value(),
            TxEnvelope::Eip2930(tx) => tx.tx().value(),
            TxEnvelope::Eip1559(tx) => tx.tx().value(),
            TxEnvelope::Eip4844(tx) => tx.tx().value(),
        }
    }

    pub fn input(&self) -> &[u8] {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().input(),
            TxEnvelope::Eip2930(tx) => tx.tx().input(),
            TxEnvelope::Eip1559(tx) => tx.tx().input(),
            TxEnvelope::Eip4844(tx) => tx.tx().input(),
        }
    }

    pub fn v(&self) -> u64 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.signature().v().to_u64(),
            TxEnvelope::Eip2930(tx) => tx.signature().v().to_u64(),
            TxEnvelope::Eip1559(tx) => tx.signature().v().to_u64(),
            TxEnvelope::Eip4844(tx) => tx.signature().v().to_u64(),
        }
    }

    pub fn r(&self) -> U256 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.signature().r(),
            TxEnvelope::Eip2930(tx) => tx.signature().r(),
            TxEnvelope::Eip1559(tx) => tx.signature().r(),
            TxEnvelope::Eip4844(tx) => tx.signature().r(),
        }
    }

    pub fn s(&self) -> U256 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.signature().s(),
            TxEnvelope::Eip2930(tx) => tx.signature().s(),
            TxEnvelope::Eip1559(tx) => tx.signature().s(),
            TxEnvelope::Eip4844(tx) => tx.signature().s(),
        }
    }

    pub fn chain_id(&self) -> Option<ChainId> {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().chain_id(),
            TxEnvelope::Eip2930(tx) => tx.tx().chain_id(),
            TxEnvelope::Eip1559(tx) => tx.tx().chain_id(),
            TxEnvelope::Eip4844(tx) => tx.tx().chain_id(),
        }
    }

    pub fn access_list(&self) -> Option<AccessList> {
        match &self.0 {
            TxEnvelope::Legacy(_) => None,
            TxEnvelope::Eip2930(tx) => Some(tx.tx().access_list.clone()),
            TxEnvelope::Eip1559(tx) => Some(tx.tx().access_list.clone()),
            TxEnvelope::Eip4844(tx) => match tx.tx() {
                TxEip4844Variant::TxEip4844(tx) => Some(tx.access_list.clone()),
                TxEip4844Variant::TxEip4844WithSidecar(tx) => Some(tx.tx().access_list.clone()),
            },
        }
    }

    pub fn max_fee_per_gas(&self) -> Option<u128> {
        match &self.0 {
            TxEnvelope::Legacy(_) => None,
            TxEnvelope::Eip2930(_) => None,
            TxEnvelope::Eip1559(tx) => Some(tx.tx().max_fee_per_gas),
            TxEnvelope::Eip4844(tx) => match tx.tx() {
                TxEip4844Variant::TxEip4844(tx) => Some(tx.max_fee_per_gas),
                TxEip4844Variant::TxEip4844WithSidecar(tx) => Some(tx.tx().max_fee_per_gas),
            },
        }
    }

    pub fn max_priority_fee_per_gas(&self) -> Option<u128> {
        match &self.0 {
            TxEnvelope::Legacy(_) => None,
            TxEnvelope::Eip2930(_) => None,
            TxEnvelope::Eip1559(tx) => Some(tx.tx().max_priority_fee_per_gas),
            TxEnvelope::Eip4844(tx) => match tx.tx() {
                TxEip4844Variant::TxEip4844(tx) => Some(tx.max_priority_fee_per_gas),
                TxEip4844Variant::TxEip4844WithSidecar(tx) => {
                    Some(tx.tx().max_priority_fee_per_gas)
                }
            },
        }
    }

    pub fn blob_versioned_hashes(&self) -> Option<Vec<FixedBytes<32>>> {
        match &self.0 {
            TxEnvelope::Legacy(_) => None,
            TxEnvelope::Eip2930(_) => None,
            TxEnvelope::Eip1559(_) => None,
            TxEnvelope::Eip4844(tx) => match tx.tx() {
                TxEip4844Variant::TxEip4844(tx) => Some(tx.blob_versioned_hashes.clone()),
                TxEip4844Variant::TxEip4844WithSidecar(tx) => {
                    Some(tx.tx().blob_versioned_hashes.clone())
                }
            },
        }
    }

    pub fn max_fee_per_blob_gas(&self) -> Option<u128> {
        match &self.0 {
            TxEnvelope::Legacy(_) => None,
            TxEnvelope::Eip2930(_) => None,
            TxEnvelope::Eip1559(_) => None,
            TxEnvelope::Eip4844(tx) => match tx.tx() {
                TxEip4844Variant::TxEip4844(tx) => Some(tx.max_fee_per_blob_gas),
                TxEip4844Variant::TxEip4844WithSidecar(tx) => Some(tx.tx().max_fee_per_blob_gas),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RpcTx(pub Transaction);

impl TryFrom<RpcTx> for ConsensusTx {
    type Error = Error;
    fn try_from(tx: RpcTx) -> Result<ConsensusTx, Error> {
        let chain_id = tx.chain_id();
        let nonce: u64 =
            tx.0.nonce
                .try_into()
                .map_err(|_| Error::ConversionError(Field::Nonce))?;
        let gas_limit: u64 =
            tx.0.gas
                .try_into()
                .map_err(|_| Error::ConversionError(Field::GasLimit))?;
        let to = tx.to();
        let value = tx.0.value;
        let input = tx.0.input.clone();
        match &tx.version()? {
            TxType::Legacy => {
                let gas_price: u128 = if let Some(gas_price) = tx.0.gas_price {
                    gas_price
                        .try_into()
                        .map_err(|_| Error::ConversionError(Field::GasPrice))?
                } else {
                    0
                };

                let res = TxLegacy {
                    chain_id,
                    nonce,
                    gas_price,
                    gas_limit,
                    to,
                    value,
                    input,
                };
                Ok(ConsensusTx(res.into_signed(tx.signature()?).into()))
            }
            TxType::Eip2930 => {
                let gas_price: u128 = if let Some(gas_price) = tx.0.gas_price {
                    gas_price
                        .try_into()
                        .map_err(|_| Error::ConversionError(Field::GasPrice))?
                } else {
                    0
                };

                let res = TxEip2930 {
                    chain_id: chain_id.unwrap(),
                    nonce,
                    gas_price,
                    gas_limit,
                    to,
                    value,
                    input,
                    access_list: tx.access_list()?,
                };
                Ok(ConsensusTx(res.into_signed(tx.signature()?).into()))
            }
            TxType::Eip1559 => {
                let max_fee_per_gas = tx.max_fee_per_gas()?;
                let max_priority_fee_per_gas = tx.max_priority_fee_per_gas()?;
                let res = TxEip1559 {
                    chain_id: chain_id.unwrap(),
                    nonce,
                    gas_limit,
                    to,
                    value,
                    input,
                    access_list: tx.access_list()?,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                };
                Ok(ConsensusTx(res.into_signed(tx.signature()?).into()))
            }
            TxType::Eip4844 => {
                let max_fee_per_gas = tx.max_fee_per_gas()?;
                let max_priority_fee_per_gas = tx.max_priority_fee_per_gas()?;
                let max_fee_per_blob_gas = tx.max_fee_per_blob_gas()?;

                let res = TxEip4844 {
                    chain_id: chain_id.unwrap(),
                    nonce,
                    gas_limit,
                    to,
                    value,
                    input,
                    access_list: tx.access_list()?,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                    max_fee_per_blob_gas,
                    blob_versioned_hashes: tx.0.blob_versioned_hashes.clone(),
                };
                Ok(ConsensusTx(res.into_signed(tx.signature()?).into()))
            }
        }
    }
}

impl RpcTx {
    fn chain_id(&self) -> Option<u64> {
        self.0
            .chain_id
            .as_ref()
            .map(|chain_id| chain_id.try_into().unwrap())
    }

    fn to(&self) -> TxKind {
        match &self.0.to {
            Some(to) => TxKind::Call(*to),
            None => TxKind::Create,
        }
    }

    fn version(&self) -> Result<TxType, Error> {
        match &self.0.transaction_type {
            Some(tx_type) if tx_type == &U64::from(0) => Ok(TxType::Legacy),
            Some(tx_type) if tx_type == &U64::from(1) => Ok(TxType::Eip2930),
            Some(tx_type) if tx_type == &U64::from(2) => Ok(TxType::Eip1559),
            Some(tx_type) if tx_type == &U64::from(3) => Ok(TxType::Eip4844),
            _ => Err(Error::InvalidTxVersion),
        }
    }

    fn max_fee_per_gas(&self) -> Result<u128, Error> {
        if let Some(value) = &self.0.max_fee_per_gas {
            Ok(value
                .try_into()
                .map_err(|_| Error::ConversionError(Field::MaxFeePerGas))?)
        } else {
            Ok(0)
        }
    }

    fn max_priority_fee_per_gas(&self) -> Result<u128, Error> {
        if let Some(value) = &self.0.max_priority_fee_per_gas {
            Ok(value
                .try_into()
                .map_err(|_| Error::ConversionError(Field::MaxPriorityFeePerGas))?)
        } else {
            Ok(0)
        }
    }

    fn max_fee_per_blob_gas(&self) -> Result<u128, Error> {
        if let Some(value) = &self.0.max_fee_per_blob_gas {
            Ok(value
                .try_into()
                .map_err(|_| Error::ConversionError(Field::MaxFeePerBlobGas))?)
        } else {
            Ok(0)
        }
    }

    fn signature(&self) -> Result<Signature, Error> {
        if let Some(signature) = self.0.signature {
            let sig = Signature::from_rs_and_parity(
                signature.r,
                signature.s,
                Parity::Eip155(
                    signature
                        .v
                        .try_into()
                        .map_err(|_| Error::ConversionError(Field::Signature))?,
                ),
            )
            .map_err(|_| Error::ConversionError(Field::Signature))?;

            Ok(sig)
        } else {
            Err(Error::ConversionError(Field::Signature))
        }
    }

    fn access_list(&self) -> Result<AccessList, Error> {
        if let Some(al) = self.0.access_list.clone() {
            let mut target_list_items: Vec<_> = vec![];
            for item in al {
                target_list_items.push(AccessListItem {
                    address: item.address,
                    storage_keys: item.storage_keys,
                });
            }
            Ok(AccessList(target_list_items))
        } else {
            Err(Error::ConversionError(Field::AccessList))
        }
    }
}
