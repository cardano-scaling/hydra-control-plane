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
                    hex::decode("03f8deb122fbbd98af8eb58ef56feda37728ec957d39586b78198a0cf624412a")
                        .expect("failed to decode prerpod hydra script reference transaction")
                        .as_slice(),
                );

                ScriptRegistry {
                    initial_reference: Input::new(tx_hash, 0).into(),
                    commit_reference: Input::new(tx_hash, 1).into(),
                    head_reference: Input::new(tx_hash, 2).into(),
                }
            }
            NetworkScriptRegistry::Mainnet => {
                let tx_hash = Hash::from(
                    hex::decode("ab1d9f8cca896bca06b70df74860deecf20774e03d8562aecaed37525f6ebead")
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
