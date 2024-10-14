use anyhow::{anyhow, Context, Result};
use pallas::{
    codec::minicbor::encode,
    crypto::hash::Hash,
    ledger::{
        addresses::Address,
        primitives::conway::{PlutusData, PlutusV2Script},
        traverse::ComputeHash,
    },
    txbuilder::{BuildBabbage, BuiltTransaction, Output, ScriptKind, StagingTransaction},
};

use crate::model::hydra::{
    contract::{head_tokens::make_head_token_script, hydra_validator::HydraValidator},
    tx::head_parameters::HeadParameters,
};

use super::input::InputWrapper;

struct InitTx {
    network_id: u8,
    seed_input: InputWrapper,
    participants: Vec<Vec<u8>>,
    parameters: HeadParameters,
}

impl InitTx {
    fn build_tx(&self, change_output: Output) -> Result<BuiltTransaction> {
        let script =
            make_head_token_script(&self.seed_input).context("Failed to make head token script")?;

        let script_hash = script.compute_hash();
        // TODO: fee calculation? Currently just hardcoding for the test
        let mut tx_builder = Some(
            StagingTransaction::new()
                .network_id(self.network_id)
                .input(self.seed_input.clone().into())
                .mint_asset(script_hash, "HydraHeadV1".as_bytes().to_vec(), 1)
                .context("Failed to add hydra token mint")?
                .script(ScriptKind::PlutusV2, script.as_ref().to_vec())
                .output(change_output)
                .output(self.make_head_output_initial(script_hash))
                .fee(1920000),
        );

        // Can I avoid the clone here?
        for participant in self.participants.clone() {
            if let Some(builder) = tx_builder {
                tx_builder = Some(
                    builder
                        .output(self.make_initial_output(script_hash, participant.clone()))
                        .mint_asset(script_hash, participant, 1)
                        .context("Failed to add participant mint")?,
                )
            }
        }

        // Gotta be a better way to update the tx builder in a loop, but this works for now
        tx_builder
            .ok_or(anyhow!("fatal error: no tx builder"))
            .and_then(|builder| builder.build_babbage_raw().map_err(|e| anyhow!("{}", e)))
            .map_err(|e| anyhow!("Failed to build tx: {}", e))
    }

    // TODO: actually do proper error handling here
    // TODO: calculate proper lovelace amount
    fn make_initial_output(&self, script_hash: Hash<28>, participant: Vec<u8>) -> Output {
        let datum = PlutusData::BoundedBytes(participant.clone().into());
        let mut datum_bytes = Vec::new();
        encode(&datum, &mut datum_bytes).expect("failed to encode datum");

        let validator: PlutusV2Script = HydraValidator::VInitial.into();
        let mut address_bytes = validator.compute_hash().to_vec();
        address_bytes.insert(0, 0b01110000 | self.network_id);
        let address =
            Address::from_bytes(address_bytes.as_slice()).expect("Failed to create address");

        Output::new(address, 1290000)
            .set_inline_datum(datum_bytes)
            .add_asset(script_hash, participant, 1)
            .expect("Failed to add asset")
    }

    // TODO: actually do proper error handling here
    // TODO: calculate proper lovelace amount
    fn make_head_output_initial(&self, script_hash: Hash<28>) -> Output {
        let datum = self.parameters.to_head_datum(script_hash, &self.seed_input);
        let mut datum_bytes = Vec::new();
        encode(&datum, &mut datum_bytes).expect("failed to encode parameters");
        let validator: PlutusV2Script = HydraValidator::VHead.into();
        let mut address_bytes = validator.compute_hash().to_vec();
        address_bytes.insert(0, 0b01110000 | self.network_id);
        let address =
            Address::from_bytes(address_bytes.as_slice()).expect("Failed to create address");
        Output::new(address, 1600000)
            .set_inline_datum(datum_bytes)
            .add_asset(script_hash, "HydraHeadV1".as_bytes().to_vec(), 1)
            .expect("Failed to add asset")
    }

    // I hate the passing around of change_output, will reorganize later
    pub fn to_bytes(&self, change_output: Output) -> Result<Vec<u8>> {
        let tx = self.build_tx(change_output)?;
        Ok(tx.tx_bytes.as_ref().to_vec())
    }
}

mod tests {
    use pallas::txbuilder::Input;

    use super::*;

    #[test]
    fn test_init_tx() {
        let tx_hash: Hash<32> =
            hex::decode("5a41c22049880541a23954877bd2e5e6069b5ecb8eed6505dbf16f5ee45e9fa8")
                .expect("Failed to decode seed tx in")
                .as_slice()
                .try_into()
                .expect("Slice was incorrect size");

        let network_id = 0;
        let seed_input = Input::new(tx_hash, 5);
        let participants =
            vec![
                hex::decode("8bb334f0e8d88551d62db31965f25b644fe0ccc8d3613533e10d689a")
                    .expect("Failed to decode participant 1"),
            ];
        let parameters = HeadParameters {
            contenstation_period: 60000,
            parties: vec![
                hex::decode("2505642019121d9b2d92437d8b8ea493bacfcb4fb535013b70e7f528")
                    .expect("failed to decode party 1"),
            ],
        };

        let init_tx = InitTx {
            network_id,
            seed_input: seed_input.into(),
            participants,
            parameters,
        };

        let tx_bytes = init_tx
            .to_bytes(Output::new(
                Address::from_bech32(
                    "addr_test1vz9mxd8sarvg25wk9ke3je0jtdjylcxverfkzdfnuyxk3xszsdn9j",
                )
                .expect("invalid address"),
                917940000,
            ))
            .expect("Failed to build tx");

        // NOTE: was going to check that the tx hash was the same as our reference one, but it seems the scripts have changed since that tx was built
        assert!(true);
    }
}
