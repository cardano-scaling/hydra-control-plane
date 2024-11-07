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
    pub fn inbound_script(&self, admin: Hash<28>) -> NativeScript {
        NativeScript::ScriptAny(vec![
            NativeScript::ScriptPubkey(admin),
            NativeScript::ScriptPubkey(self.signing_key),
        ])
    }
    pub fn outbound_script(&self, admin: Hash<28>) -> NativeScript {
        NativeScript::ScriptAny(vec![
            NativeScript::ScriptPubkey(self.signing_key),
            NativeScript::ScriptPubkey(admin),
        ])
    }

    pub fn outbound_address(&self, admin: Hash<28>, network: Network) -> Result<Address> {
        let native_script: NativeScript = self.outbound_script(admin);
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

    pub fn inbound_address(&self, admin: Hash<28>, network: Network) -> Result<Address> {
        let native_script: NativeScript = self.inbound_script(admin);
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
