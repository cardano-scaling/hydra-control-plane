use std::ops::Deref;

use pallas::{
    ledger::primitives::{
        alonzo,
        conway::{Constr, PlutusData},
    },
    txbuilder::Input,
};

pub struct InputWrapper {
    pub inner: Input,
}

impl InputWrapper {
    pub fn to_plutus_data(&self) -> PlutusData {
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: vec![PlutusData::BoundedBytes(alonzo::BoundedBytes::from(
                        self.inner.tx_hash.0.to_vec(),
                    ))],
                }),
                PlutusData::BigInt(alonzo::BigInt::Int((self.inner.txo_index as i64).into())),
            ],
        })
    }
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
