use crate::{Error, Field};
use alloy::consensus::{
    SignableTransaction, TxEip1559, TxEip2930, TxEip4844, TxEnvelope, TxLegacy, TxType,
};
use alloy::consensus::{Transaction as ConsensusTransaction, TxEip4844Variant};
use alloy::eips::eip2718::Decodable2718;
use alloy::eips::eip2930::AccessList;
use alloy::eips::eip2930::AccessListItem;
use alloy::network::eip2718::Encodable2718;
use alloy::primitives::{ChainId, FixedBytes, Parity, Signature, TxKind, U256};
use alloy::rpc::types::Transaction;

#[derive(Debug, Clone)]
pub struct ConsensusTx(pub TxEnvelope);

impl ConsensusTx {
    pub fn rlp_encode(&self) -> Vec<u8> {
        self.0.encoded_2718()
    }

    pub fn rlp_decode(mut data: &[u8]) -> Result<Self, Error> {
        let tx = TxEnvelope::decode_2718(&mut data).map_err(Error::Eip)?;
        Ok(ConsensusTx(tx))
    }

    pub fn nonce(&self) -> u64 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().nonce(),
            TxEnvelope::Eip2930(tx) => tx.tx().nonce(),
            TxEnvelope::Eip1559(tx) => tx.tx().nonce(),
            TxEnvelope::Eip4844(tx) => tx.tx().nonce(),
            _ => todo!(),
        }
    }

    pub fn gas_limit(&self) -> u128 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().gas_limit(),
            TxEnvelope::Eip2930(tx) => tx.tx().gas_limit(),
            TxEnvelope::Eip1559(tx) => tx.tx().gas_limit(),
            TxEnvelope::Eip4844(tx) => tx.tx().gas_limit(),
            _ => todo!(),
        }
    }

    pub fn gas_price(&self) -> Option<u128> {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().gas_price(),
            TxEnvelope::Eip2930(tx) => tx.tx().gas_price(),
            TxEnvelope::Eip1559(tx) => tx.tx().gas_price(),
            TxEnvelope::Eip4844(tx) => tx.tx().gas_price(),
            _ => todo!(),
        }
    }

    pub fn to(&self) -> TxKind {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().to(),
            TxEnvelope::Eip2930(tx) => tx.tx().to(),
            TxEnvelope::Eip1559(tx) => tx.tx().to(),
            TxEnvelope::Eip4844(tx) => tx.tx().to(),
            _ => todo!(),
        }
    }

    pub fn value(&self) -> U256 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().value(),
            TxEnvelope::Eip2930(tx) => tx.tx().value(),
            TxEnvelope::Eip1559(tx) => tx.tx().value(),
            TxEnvelope::Eip4844(tx) => tx.tx().value(),
            _ => todo!(),
        }
    }

    pub fn input(&self) -> &[u8] {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().input(),
            TxEnvelope::Eip2930(tx) => tx.tx().input(),
            TxEnvelope::Eip1559(tx) => tx.tx().input(),
            TxEnvelope::Eip4844(tx) => tx.tx().input(),
            _ => todo!(),
        }
    }

    pub fn v(&self) -> u64 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.signature().v().to_u64(),
            TxEnvelope::Eip2930(tx) => tx.signature().v().to_u64(),
            TxEnvelope::Eip1559(tx) => tx.signature().v().to_u64(),
            TxEnvelope::Eip4844(tx) => tx.signature().v().to_u64(),
            _ => todo!(),
        }
    }

    pub fn r(&self) -> U256 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.signature().r(),
            TxEnvelope::Eip2930(tx) => tx.signature().r(),
            TxEnvelope::Eip1559(tx) => tx.signature().r(),
            TxEnvelope::Eip4844(tx) => tx.signature().r(),
            _ => todo!(),
        }
    }

    pub fn s(&self) -> U256 {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.signature().s(),
            TxEnvelope::Eip2930(tx) => tx.signature().s(),
            TxEnvelope::Eip1559(tx) => tx.signature().s(),
            TxEnvelope::Eip4844(tx) => tx.signature().s(),
            _ => todo!(),
        }
    }

    pub fn sender(&self) -> Result<alloy::primitives::Address, alloy::primitives::SignatureError> {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.recover_signer(),
            TxEnvelope::Eip2930(tx) => tx.recover_signer(),
            TxEnvelope::Eip1559(tx) => tx.recover_signer(),
            TxEnvelope::Eip4844(tx) => tx.recover_signer(),
            _ => todo!(),
        }
    }

    pub fn chain_id(&self) -> Option<ChainId> {
        match &self.0 {
            TxEnvelope::Legacy(tx) => tx.tx().chain_id(),
            TxEnvelope::Eip2930(tx) => tx.tx().chain_id(),
            TxEnvelope::Eip1559(tx) => tx.tx().chain_id(),
            TxEnvelope::Eip4844(tx) => tx.tx().chain_id(),
            _ => todo!(),
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
            _ => todo!(),
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
            _ => todo!(),
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
            _ => todo!(),
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
            _ => todo!(),
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
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RpcTx(pub Transaction);

impl TryFrom<RpcTx> for ConsensusTx {
    type Error = Error;
    fn try_from(tx: RpcTx) -> Result<ConsensusTx, Error> {
        let chain_id = tx.chain_id();
        let nonce: u64 = tx.0.nonce;
        let gas_limit: u128 = tx.0.gas;

        let value = tx.0.value;
        let input = tx.0.input.clone();
        match &tx.version()? {
            TxType::Legacy => {
                let to = tx.to();
                let gas_price: u128 = tx.0.gas_price.unwrap_or_default();

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
                let to = tx.to();
                let gas_price: u128 = tx.0.gas_price.unwrap_or_default();

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
                let to = tx.to();
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
                let to = match tx.to() {
                    TxKind::Call(to) => to,
                    TxKind::Create => return Err(Error::InvalidTxVersion),
                };
                let blob_versioned_hashes = tx
                    .clone()
                    .0
                    .blob_versioned_hashes
                    .ok_or(Error::ConversionError(Field::Input))?;
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
                    blob_versioned_hashes,
                };
                Ok(ConsensusTx(res.into_signed(tx.signature()?).into()))
            }
        }
    }
}

impl RpcTx {
    fn chain_id(&self) -> Option<u64> {
        self.0.chain_id
    }

    fn to(&self) -> TxKind {
        match self.0.to {
            Some(to) => TxKind::Call(to),
            None => TxKind::Create,
        }
    }

    fn version(&self) -> Result<TxType, Error> {
        match self.0.transaction_type {
            Some(0) => Ok(TxType::Legacy),
            Some(1) => Ok(TxType::Eip2930),
            Some(2) => Ok(TxType::Eip1559),
            Some(3) => Ok(TxType::Eip4844),
            None => Ok(TxType::Legacy),
            _ => Err(Error::InvalidTxVersion),
        }
    }

    fn max_fee_per_gas(&self) -> Result<u128, Error> {
        if let Some(value) = self.0.max_fee_per_gas {
            Ok(value)
        } else {
            Ok(0)
        }
    }

    fn max_priority_fee_per_gas(&self) -> Result<u128, Error> {
        if let Some(value) = self.0.max_priority_fee_per_gas {
            Ok(value)
        } else {
            Ok(0)
        }
    }

    fn max_fee_per_blob_gas(&self) -> Result<u128, Error> {
        if let Some(value) = self.0.max_fee_per_blob_gas {
            Ok(value)
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
            let target_list_items: Vec<AccessListItem> = Vec::<AccessListItem>::from(al);
            Ok(AccessList(target_list_items))
        } else {
            Err(Error::ConversionError(Field::AccessList))
        }
    }
}
