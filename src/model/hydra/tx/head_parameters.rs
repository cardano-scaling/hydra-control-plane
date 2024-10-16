use pallas::{
    crypto::hash::Hash,
    ledger::primitives::{
        alonzo,
        conway::{Constr, PlutusData},
    },
};

use super::input::InputWrapper;

// Note: I am using i64 to avoid using `as` type casts. BigInt requires an i64, although typically they should be u64.
pub struct HeadParameters {
    pub contenstation_period: i64, // Number of seconds
    pub parties: Vec<Vec<u8>>,     // VerificationKeyHash
}

impl HeadParameters {
    pub fn to_head_datum(
        &self,
        token_policy_id: Hash<28>,
        seed_tx_in: &InputWrapper,
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
                PlutusData::BoundedBytes(alonzo::BoundedBytes::from(token_policy_id.to_vec())),
                seed_tx_in.into(),
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use pallas::{codec::minicbor::encode, txbuilder::Input};

    use super::*;

    #[test]
    // This test is testing against the datum in the following tx: bc33420ff8560e29b9172c1bec01f5fe6299cae4c718e35f9853e4be9b2a4b9c
    fn builds_expected_datum() {
        let particpant_key =
            hex::decode("b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb")
                .expect("Failed to decode participant key");

        let token_policy_id: Hash<28> = Hash::new(
            hex::decode("983a93519f98636e38f2d8050f4f66c046bca4be38b06384a2fd6cd6")
                .expect("Failed to decode token policy id")
                .as_slice()
                .try_into()
                .expect("Slice was incorrect size"),
        );

        let head_parameters = HeadParameters {
            contenstation_period: 60000,
            parties: vec![particpant_key],
        };

        let tx_hash: Hash<32> =
            hex::decode("800a656c030ed34c071598f5beb361494b88092011fa4895578d820aadba397d")
                .expect("Failed to decode seed tx in")
                .as_slice()
                .try_into()
                .expect("Slice was incorrect size");

        let seed_tx_in = InputWrapper::from(Input::new(tx_hash, 0));

        let datum = head_parameters.to_head_datum(token_policy_id, &seed_tx_in);
        let mut datum_bytes: Vec<u8> = Vec::new();
        encode(&datum, &mut datum_bytes).expect("Failed to encode datum");
        assert_eq!(hex::encode(datum_bytes), "d8799fd8799f19ea60ff9f5820b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707ebff581c983a93519f98636e38f2d8050f4f66c046bca4be38b06384a2fd6cd6d8799fd8799f5820800a656c030ed34c071598f5beb361494b88092011fa4895578d820aadba397dff00ffff".to_string())
    }
}
