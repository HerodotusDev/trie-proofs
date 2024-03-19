use crate::Error;
use alloy_consensus::{
    SignableTransaction, TxEip1559, TxEip2930, TxEip4844, TxEnvelope, TxLegacy, TxType,
};
use alloy_eips::eip2930::AccessList;
use alloy_eips::eip2930::AccessListItem;
use alloy_primitives::{Parity, Signature, TxKind, U64};
use alloy_rpc_types::Transaction;

#[derive(Debug, Clone)]
pub struct ConsensusTx(pub TxEnvelope);
#[derive(Debug, Clone)]
pub struct RpcTx(pub Transaction);
impl TryFrom<RpcTx> for ConsensusTx {
    type Error = Error;
    fn try_from(tx: RpcTx) -> Result<ConsensusTx, Error> {
        let chain_id = tx.chain_id().unwrap();
        let nonce: u64 = tx.0.nonce.try_into().unwrap();
        let gas_limit: u64 = tx.0.gas.try_into().unwrap();
        let to = tx.to();
        let value = tx.0.value;
        let input = tx.0.input.clone();

        match &tx.version()? {
            TxType::Legacy => {
                let gas_price: u128 = if let Some(gas_price) = tx.0.gas_price {
                    gas_price.try_into().unwrap()
                } else {
                    0
                };

                let res = TxLegacy {
                    chain_id: Some(chain_id),
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
                    gas_price.try_into().map_err(|_| Error::ConversionError)?
                } else {
                    0
                };

                let res = TxEip2930 {
                    chain_id,
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
                    chain_id,
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
                let max_fee_per_gas = tx.max_fee_per_blob_gas()?;
                let max_priority_fee_per_gas = tx.max_priority_fee_per_gas()?;
                let max_fee_per_blob_gas = tx.max_fee_per_blob_gas()?;

                let res = TxEip4844 {
                    chain_id,
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
            Ok(value.try_into().map_err(|_| Error::ConversionError)?)
        } else {
            Ok(0)
        }
    }

    fn max_priority_fee_per_gas(&self) -> Result<u128, Error> {
        if let Some(value) = &self.0.max_priority_fee_per_gas {
            Ok(value.try_into().map_err(|_| Error::ConversionError)?)
        } else {
            Ok(0)
        }
    }

    fn max_fee_per_blob_gas(&self) -> Result<u128, Error> {
        if let Some(value) = &self.0.max_fee_per_blob_gas {
            Ok(value.try_into().map_err(|_| Error::ConversionError)?)
        } else {
            Ok(0)
        }
    }

    fn signature(&self) -> Result<Signature, Error> {
        if let Some(signature) = self.0.signature {
            let sig = Signature::from_rs_and_parity(
                signature.r,
                signature.s,
                Parity::Eip155(signature.v.try_into().map_err(|_| Error::ConversionError)?),
            )
            .map_err(|_| Error::ConversionError)?;

            Ok(sig)
        } else {
            Err(Error::ConversionError)
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
            Err(Error::ConversionError)
        }
    }
}
