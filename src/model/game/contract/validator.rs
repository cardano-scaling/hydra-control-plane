use pallas::ledger::{
    addresses::{Address, Network},
    primitives::PlutusScript,
    traverse::ComputeHash,
};

pub struct Validator {}

impl Validator {
    pub fn cbor() -> String {
        include_str!("_referee.cbor").trim_end().to_string()
    }

    pub fn address(network: Network) -> Address {
        let mut hash = Self::to_plutus().compute_hash().to_vec();
        hash.insert(
            0,
            0b01110000
                | match network {
                    Network::Testnet => 0,
                    Network::Mainnet => 1,
                    Network::Other(i) => i,
                },
        );

        Address::from_bytes(hash.as_slice()).expect("Failed to create address for a script")
    }

    pub fn to_plutus() -> PlutusScript<3> {
        PlutusScript(
            hex::decode(Self::cbor())
                .expect("invalid script cbor hex string")
                .into(),
        )
    }
}
