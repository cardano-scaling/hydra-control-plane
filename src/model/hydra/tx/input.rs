use std::ops::Deref;

use pallas::{
    codec::utils::MaybeIndefArray,
    crypto::hash::Hash,
    ledger::primitives::{
        alonzo,
        conway::{Constr, PlutusData},
    },
    txbuilder::Input,
};

pub struct InputWrapper {
    pub inner: Input,
}

impl Deref for InputWrapper {
    type Target = Input;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Input> for InputWrapper {
    fn from(value: Input) -> Self {
        Self { inner: value }
    }
}

impl From<InputWrapper> for Input {
    fn from(value: InputWrapper) -> Self {
        value.inner
    }
}

impl TryFrom<String> for InputWrapper {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let hash_index = value.split("#").collect::<Vec<&str>>();
        let index = hash_index[1].parse::<u64>()?;
        let hash_bytes = hex::decode(hash_index[0])?;
        let hash = hash_bytes.as_slice();

        Ok(Input::new(Hash::from(hash), index).into())
    }
}

impl Into<PlutusData> for InputWrapper {
    fn into(self) -> PlutusData {
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: MaybeIndefArray::Indef(vec![
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: MaybeIndefArray::Indef(vec![PlutusData::BoundedBytes(
                        alonzo::BoundedBytes::from(self.inner.tx_hash.0.to_vec()),
                    )]),
                }),
                PlutusData::BigInt(alonzo::BigInt::Int((self.inner.txo_index as i64).into())),
            ]),
        })
    }
}
impl Into<PlutusData> for &InputWrapper {
    fn into(self) -> PlutusData {
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: MaybeIndefArray::Indef(vec![
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: MaybeIndefArray::Indef(vec![PlutusData::BoundedBytes(
                        alonzo::BoundedBytes::from(self.inner.tx_hash.0.to_vec()),
                    )]),
                }),
                PlutusData::BigInt(alonzo::BigInt::Int((self.inner.txo_index as i64).into())),
            ]),
        })
    }
}

impl Clone for InputWrapper {
    fn clone(&self) -> Self {
        Self {
            inner: Input::new(self.inner.tx_hash.0.into(), self.inner.txo_index),
        }
    }
}
