use anyhow::Result;
use pallas::crypto::hash::Hash;
use pallas::ledger::addresses::{Address, Network, PaymentKeyHash};
use pallas::ledger::primitives::conway::NativeScript;
use pallas::ledger::traverse::ComputeHash;

use super::contract::game_state::PaymentCredential;
use super::contract::validator::Validator;

pub struct Player {
    pub signing_key: Hash<28>,
}

impl Player {
    pub fn new(signing_key: Hash<28>) -> Self {
        Self { signing_key }
    }
    pub fn inbound_script(&self) -> NativeScript {
        NativeScript::ScriptAny(vec![
            NativeScript::ScriptPubkey(Validator::compute_hash().as_slice().into()),
            NativeScript::ScriptPubkey(self.signing_key),
        ])
    }
    pub fn outbound_script(&self) -> NativeScript {
        NativeScript::ScriptAny(vec![
            NativeScript::ScriptPubkey(self.signing_key),
            NativeScript::ScriptPubkey(Validator::compute_hash().as_slice().into()),
        ])
    }

    pub fn outbound_address(&self, network: Network) -> Result<Address> {
        let native_script: NativeScript = self.outbound_script();
        let mut bytes = native_script.compute_hash().to_vec();
        bytes.insert(
            0,
            0b01110000
                | match network {
                    Network::Testnet => 0,
                    Network::Mainnet => 1,
                    Network::Other(i) => i,
                },
        );

        Address::from_bytes(bytes.as_slice()).map_err(anyhow::Error::msg)
    }

    pub fn inbound_address(&self, network: Network) -> Result<Address> {
        let native_script: NativeScript = self.inbound_script();
        let mut bytes = native_script.compute_hash().to_vec();
        bytes.insert(
            0,
            0b01110000
                | match network {
                    Network::Testnet => 0,
                    Network::Mainnet => 1,
                    Network::Other(i) => i,
                },
        );

        Address::from_bytes(bytes.as_slice()).map_err(anyhow::Error::msg)
    }
}

impl From<PaymentKeyHash> for Player {
    fn from(value: PaymentKeyHash) -> Self {
        Self { signing_key: value }
    }
}

impl From<PaymentCredential> for Player {
    fn from(value: PaymentCredential) -> Self {
        let pkh: Hash<28> = value.into();
        Self { signing_key: pkh }
    }
}
