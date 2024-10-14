use pallas::ledger::primitives::conway::PlutusV2Script;

// Named after the scripts here: https://github.com/cardano-scaling/hydra/tree/master/hydra-plutus/scripts
// For more info on what each script does, read the protocol spec: https://hydra.family/head-protocol/assets/files/hydra-spec-74c85a9e8c75aeca7735137947b39453.pdf
pub enum HydraValidator {
    MHead,
    VDeposit,
    VHead,
    VInitial,
}

impl HydraValidator {
    pub fn cbor(&self) -> &str {
        match self {
            Self::MHead => include_str!("_mhead.cbor").trim_end(),
            Self::VDeposit => include_str!("_vdeposit.cbor").trim_end(),
            Self::VHead => include_str!("_vhead.cbor").trim_end(),
            Self::VInitial => include_str!("_vinitial.cbor").trim_end(),
        }
    }

    pub fn to_plutus(&self) -> PlutusV2Script {
        PlutusV2Script(
            hex::decode(self.cbor())
                .expect("invalid script cbor hex string")
                .into(),
        )
    }
}

// I feel OK with an expect here, as if we have invalid script cbor encoding, it's because we have a bug in the codebase
impl Into<PlutusV2Script> for HydraValidator {
    fn into(self) -> PlutusV2Script {
        PlutusV2Script(
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
