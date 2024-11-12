use std::ops::Deref;

use pallas::{
    codec::utils::MaybeIndefArray,
    ledger::{
        addresses::{Address, ShelleyDelegationPart, ShelleyPaymentPart},
        primitives::conway::{BigInt, Constr, PlutusData},
    },
    txbuilder::Output,
};

#[derive(Debug, Clone)]
pub struct OutputWrapper {
    pub inner: Output,
}

impl OutputWrapper {
    fn payment_key_hash(&self) -> Vec<u8> {
        match &self.inner.address.0 {
            Address::Byron(byron) => byron.payload.0.deref().clone(),
            Address::Shelley(shelley) => match shelley.payment() {
                ShelleyPaymentPart::Key(key) => key.deref().to_vec(),
                ShelleyPaymentPart::Script(script) => script.deref().to_vec(),
            },
            // An output can never be at a stake address, this should panic
            Address::Stake(_) => panic!("Unreachable code"),
        }
    }

    fn delegation_key_hash(&self) -> Option<Vec<u8>> {
        match &self.inner.address.0 {
            Address::Shelley(shelley) => match shelley.delegation() {
                ShelleyDelegationPart::Key(key) => Some(key.deref().to_vec()),
                ShelleyDelegationPart::Script(script) => Some(script.deref().to_vec()),
                // No need to support pointer addresses as they aren't used
                ShelleyDelegationPart::Pointer(_) => None,
                ShelleyDelegationPart::Null => None,
            },
            _ => None,
        }
    }

    fn assets_to_plutus_data(&self) -> PlutusData {
        let asset_map = vec![(
            PlutusData::BoundedBytes(vec![].into()),
            PlutusData::Map(
                vec![(
                    PlutusData::BoundedBytes(vec![].into()),
                    PlutusData::BigInt(BigInt::Int((self.inner.lovelace as i64).into())),
                )]
                .into(),
            ),
        )];

        match &self.inner.assets {
            None => PlutusData::Map(asset_map.into()),
            Some(assets) => PlutusData::Map(
                [
                    asset_map.as_slice(),
                    assets
                        .iter()
                        .map(|(k, v)| {
                            (
                                PlutusData::BoundedBytes(k.0.to_vec().into()),
                                PlutusData::Map(
                                    v.into_iter()
                                        .map(|(k, v)| {
                                            (
                                                PlutusData::BoundedBytes(k.0.to_vec().into()),
                                                PlutusData::BigInt(BigInt::Int(
                                                    (v.clone() as i64).into(),
                                                )),
                                            )
                                        })
                                        .collect::<Vec<(_, _)>>()
                                        .into(),
                                ),
                            )
                        })
                        .collect::<Vec<(_, _)>>()
                        .as_slice(),
                ]
                .concat()
                .into(),
            ),
        }
    }
}

impl From<Output> for OutputWrapper {
    fn from(value: Output) -> Self {
        Self { inner: value }
    }
}

impl Into<PlutusData> for OutputWrapper {
    fn into(self) -> PlutusData {
        // Output Object
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: MaybeIndefArray::Indef(vec![
                // Address Object
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: MaybeIndefArray::Indef(vec![
                        // Payment Part
                        PlutusData::Constr(Constr {
                            tag: 121,
                            any_constructor: None,
                            fields: MaybeIndefArray::Indef(vec![PlutusData::BoundedBytes(
                                self.payment_key_hash().into(),
                            )]),
                        }),
                        // Maybe<Delegation Part>
                        PlutusData::Constr(match self.delegation_key_hash() {
                            // NOTE: this case may not be correct. Might need another wrapper, but unclear.
                            Some(stake_key_hash) => Constr {
                                tag: 121,
                                any_constructor: None,
                                fields: MaybeIndefArray::Indef(vec![PlutusData::BoundedBytes(
                                    stake_key_hash.into(),
                                )]),
                            },
                            None => Constr {
                                tag: 122,
                                any_constructor: None,
                                fields: MaybeIndefArray::Def(vec![]),
                            },
                        }),
                    ]),
                }),
                // Value
                self.assets_to_plutus_data(),
                // Datum
                // TODO: figure out expected encoding for datum variants besised None
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: MaybeIndefArray::Def(vec![]),
                }),
                // Script Ref
                // TODO: figure out expected encoding for script ref variant besides None
                PlutusData::Constr(Constr {
                    tag: 122,
                    any_constructor: None,
                    fields: MaybeIndefArray::Def(vec![]),
                }),
            ]),
        })
    }
}

impl Deref for OutputWrapper {
    type Target = Output;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use pallas::codec::minicbor::encode;

    use super::*;

    #[test]
    fn test_output_to_plutus_data() {
        let output: OutputWrapper = Output::new(
            Address::from_bech32(
                "addr_test1qq5ev5x5x808whr9amg4cy32496jxljmfg345ktvpd0deu6fd2eq8xrhkcuxve4r6eg78r40qnrupfrvp8mljw0tl4hqe383dk"
            )
            .expect("failed to decode bech32"),
            10000000
        ).into();

        let plutus_data: PlutusData = output.into();
        let mut bytes = Vec::new();
        encode(&plutus_data, &mut bytes).expect("failed to encode plutus data");

        assert_eq!(hex::encode(bytes), "D8799FD8799FD8799F581C299650D431DE775C65EED15C122AA975237E5B4A235A596C0B5EDCF3FFD8799FD8799FD8799F581C496AB2039877B6386666A3D651E38EAF04C7C0A46C09F7F939EBFD6EFFFFFFFFA140A1401A00989680D87980D87A80FF".to_lowercase())
    }
}
