use pallas::ledger::{addresses::Address, primitives::PlutusScript, traverse::ComputeHash};

// Named after the scripts here: https://github.com/cardano-scaling/hydra/tree/master/hydra-plutus/scripts
// For more info on what each script does, read the protocol spec: https://hydra.family/head-protocol/assets/files/hydra-spec-74c85a9e8c75aeca7735137947b39453.pdf
pub enum HydraValidator {
    MHead,
    VCommit,
    VDeposit,
    VHead,
    VInitial,
}

impl HydraValidator {
    pub fn cbor(&self) -> &str {
        match self {
            Self::MHead => include_str!("_mhead.cbor").trim_end(),
            Self::VCommit => include_str!("_vcommit.cbor").trim_end(),
            Self::VDeposit => include_str!("_vdeposit.cbor").trim_end(),
            Self::VHead => include_str!("_vhead.cbor").trim_end(),
            Self::VInitial => include_str!("_vinitial.cbor").trim_end(),
        }
    }

    pub fn to_plutus(&self) -> PlutusScript<2> {
        PlutusScript(
            hex::decode(self.cbor())
                .expect("invalid script cbor hex string")
                .into(),
        )
    }

    pub fn to_address(&self, network_id: u8) -> Address {
        let mut hash = self.to_plutus().compute_hash().to_vec();
        hash.insert(0, 0b01110000 | network_id);

        Address::from_bytes(hash.as_slice()).expect("Failed to create address for a script")
    }
}

// I feel OK with an expect here, as if we have invalid script cbor encoding, it's because we have a bug in the codebase
impl Into<PlutusScript<2>> for HydraValidator {
    fn into(self) -> PlutusScript<2> {
        PlutusScript(
            hex::decode(self.cbor())
                .expect("invalid script cbor hex string")
                .into(),
        )
    }
}

impl Into<Vec<u8>> for HydraValidator {
    fn into(self) -> Vec<u8> {
        hex::decode(self.cbor()).expect("invalid script cbor hex string")
    }
}
