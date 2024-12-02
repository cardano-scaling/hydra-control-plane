use pallas::{crypto::hash::Hash, txbuilder::Input};

use super::input::InputWrapper;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ScriptRegistry {
    pub initial_reference: InputWrapper,
    pub commit_reference: InputWrapper,
    pub head_reference: InputWrapper,
}

#[allow(dead_code)]
pub enum NetworkScriptRegistry {
    Mainnet,
    Preview,
    Preprod,
}

impl From<NetworkScriptRegistry> for ScriptRegistry {
    fn from(value: NetworkScriptRegistry) -> Self {
        match value {
            NetworkScriptRegistry::Preprod => {
                let tx_hash = Hash::from(
                    hex::decode("f41e346809f765fb161f060b3e40fac318c361f1be29bd2b827d46d765195e93")
                        .expect("failed to decode prerpod hydra script reference transaction")
                        .as_slice(),
                );

                ScriptRegistry {
                    initial_reference: Input::new(tx_hash, 0).into(),
                    commit_reference: Input::new(tx_hash, 1).into(),
                    head_reference: Input::new(tx_hash, 2).into(),
                }
            }
            _ => panic!("Unimplemented"),
        }
    }
}
