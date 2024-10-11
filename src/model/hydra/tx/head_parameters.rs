use pallas::ledger::primitives::{
    alonzo,
    conway::{Constr, PlutusData},
};

// Note: I am using i64 to avoid using `as` type casts. BigInt requires an i64, although typically they should be u64.
pub struct HeadParameters {
    contenstation_period: i64, // Number of seconds
    parties: Vec<Vec<u8>>,     // VerificationKey
}

impl HeadParameters {
    pub fn to_head_datum(
        &self,
        token_policy_id: Vec<u8>,
        seed_tx_in: (Vec<u8>, i64),
    ) -> PlutusData {
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: vec![PlutusData::BigInt(alonzo::BigInt::Int(
                        self.contenstation_period.into(),
                    ))],
                }),
                PlutusData::Array(
                    self.parties
                        .iter()
                        .map(|v| PlutusData::BoundedBytes(alonzo::BoundedBytes::from(v.clone())))
                        .collect(),
                ),
                PlutusData::BoundedBytes(alonzo::BoundedBytes::from(token_policy_id)),
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: vec![
                        PlutusData::Constr(Constr {
                            tag: 121,
                            any_constructor: None,
                            fields: vec![PlutusData::BoundedBytes(alonzo::BoundedBytes::from(
                                seed_tx_in.0,
                            ))],
                        }),
                        PlutusData::BigInt(alonzo::BigInt::Int(seed_tx_in.1.into())),
                    ],
                }),
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use pallas::codec::minicbor::encode;

    use super::*;

    #[test]
    // This test is testing against the datum in the following tx: bc33420ff8560e29b9172c1bec01f5fe6299cae4c718e35f9853e4be9b2a4b9c
    fn builds_expected_datum() {
        let particpant_key =
            hex::decode("b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb")
                .expect("Failed to decode participant key");

        let token_policy_id =
            hex::decode("983a93519f98636e38f2d8050f4f66c046bca4be38b06384a2fd6cd6")
                .expect("Failed to decode token policy id");

        let head_parameters = HeadParameters {
            contenstation_period: 60000,
            parties: vec![particpant_key],
        };

        let seed_tx_in = (
            hex::decode("800a656c030ed34c071598f5beb361494b88092011fa4895578d820aadba397d")
                .expect("Failed to decode seed tx in"),
            0,
        );
        let datum = head_parameters.to_head_datum(token_policy_id, seed_tx_in);
        let mut datum_bytes: Vec<u8> = Vec::new();
        encode(&datum, &mut datum_bytes).expect("Failed to encode datum");
        assert_eq!(hex::encode(datum_bytes), "d8799fd8799f19ea60ff9f5820b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707ebff581c983a93519f98636e38f2d8050f4f66c046bca4be38b06384a2fd6cd6d8799fd8799f5820800a656c030ed34c071598f5beb361494b88092011fa4895578d820aadba397dff00ffff".to_string())
    }
}
