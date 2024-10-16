use anyhow::{Context, Result};

use pallas::{
    codec::minicbor::encode,
    ledger::{
        addresses::PaymentKeyHash,
        primitives::conway::{Constr, PlutusData},
    },
    txbuilder::{Output, StagingTransaction},
};

use crate::model::hydra::contract::hydra_validator::HydraValidator;

use super::{input::InputWrapper, script_registry::ScriptRegistry};

pub struct CommitTx {
    network_id: u8,
    script_registry: ScriptRegistry,
    head_id: Vec<u8>,
    party: Vec<u8>,
    initial_input: (InputWrapper, Output, PaymentKeyHash),
    commit_inputs: Vec<(InputWrapper, Output)>,
}

impl CommitTx {
    pub fn build_tx(&self) -> Result<StagingTransaction> {
        let commit_output = build_base_commit_output(
            self.commit_inputs.iter().map(|(_, o)| o.clone()).collect(),
            self.network_id,
        )
        .context("Failed to construct base commit output")?
        .set_inline_datum(self.make_commit_datum()?);

        let tx_builder = StagingTransaction::new().output(commit_output);

        Ok(tx_builder)
    }

    fn make_commit_datum(&self) -> Result<Vec<u8>> {
        let data = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                PlutusData::BoundedBytes(self.party.clone().into()),
                PlutusData::Array(
                    self.commit_inputs
                        .clone()
                        .into_iter()
                        .map(|commit_input| commit_input.0.into())
                        .collect(),
                ),
            ],
        });

        let mut bytes: Vec<u8> = Vec::new();
        encode(&data, &mut bytes).context("Failed to encode plutus data in CBOR")?;

        Ok(bytes)
    }
}

fn build_base_commit_output(outputs: Vec<Output>, network_id: u8) -> Result<Output> {
    let address = HydraValidator::VDeposit.to_address(network_id);
    let lovelace = outputs.iter().fold(0, |acc, o| acc + o.lovelace);
    let mut commit_output = Output::new(address, lovelace);
    for output in outputs {
        if let Some(output_assets) = output.assets {
            for (policy, assets) in output_assets.iter() {
                for (name, amount) in assets {
                    commit_output = commit_output
                        .add_asset(policy.0.into(), name.0.clone(), amount.clone())
                        .context("Failed to add asset to commit output")?;
                }
            }
        }
    }

    Ok(commit_output)
}
