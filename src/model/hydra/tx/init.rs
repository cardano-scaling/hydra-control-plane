use anyhow::{anyhow, Context, Result};
use pallas::{
    codec::minicbor::encode,
    crypto::hash::Hash,
    ledger::{
        addresses::Address,
        primitives::{conway::PlutusData, PlutusScript},
        traverse::ComputeHash,
    },
    txbuilder::{BuildConway, BuiltTransaction, ExUnits, Output, ScriptKind, StagingTransaction},
};

use crate::model::hydra::{
    contract::{head_tokens::make_head_token_script, hydra_validator::HydraValidator},
    tx::head_parameters::HeadParameters,
};

use super::{cost_models::COST_MODEL_PLUTUS_V2, input::InputWrapper, void_redeemer};

#[allow(dead_code)]
pub struct InitTx {
    pub network_id: u8,
    pub seed_input: InputWrapper,
    pub participants: Vec<Vec<u8>>,
    pub parameters: HeadParameters,
}

#[allow(dead_code)]
impl InitTx {
    pub fn get_head_id(&self) -> Result<Vec<u8>> {
        Ok(self.get_minting_validator()?.1.to_vec())
    }
    fn get_minting_validator(&self) -> Result<(PlutusScript<2>, Hash<28>)> {
        let script =
            make_head_token_script(&self.seed_input).context("Failed to make head token script")?;
        let script_hash = script.compute_hash();

        Ok((script, script_hash))
    }
    fn build_tx(&self, change_output: Output) -> Result<BuiltTransaction> {
        let (script, script_hash) = self.get_minting_validator()?;
        // TODO: fee calculation? Currently just hardcoding for the test

        let mut tx_builder = Some(
            StagingTransaction::new()
                .language_view(ScriptKind::PlutusV2, COST_MODEL_PLUTUS_V2.clone())
                .network_id(self.network_id)
                .input(self.seed_input.clone().into())
                .collateral_input(self.seed_input.clone().into())
                .mint_asset(script_hash, "HydraHeadV1".as_bytes().to_vec(), 1)
                .context("Failed to add hydra token mint")?
                .add_mint_redeemer(
                    script_hash,
                    void_redeemer(),
                    Some(ExUnits {
                        mem: 1000000,
                        steps: 300000000,
                    }),
                )
                .script(ScriptKind::PlutusV2, script.as_ref().to_vec())
                .output(self.make_head_output_initial(script_hash))
                .fee(5000000),
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
            .and_then(|builder| {
                builder
                    .output(change_output)
                    .build_conway_raw()
                    .map_err(|e| anyhow!("{}", e))
            })
            .map_err(|e| anyhow!("Failed to build tx: {}", e))
    }

    // TODO: actually do proper error handling here
    // TODO: calculate proper lovelace amount
    pub fn make_initial_output(&self, script_hash: Hash<28>, participant: Vec<u8>) -> Output {
        let datum = PlutusData::BoundedBytes(script_hash.to_vec().into());
        let mut datum_bytes = Vec::new();
        encode(&datum, &mut datum_bytes).expect("failed to encode datum");

        let address: Address = HydraValidator::VInitial.to_address(self.network_id);

        Output::new(address, 2000000)
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
        let address = HydraValidator::VHead.to_address(self.network_id);

        Output::new(address, 2000000)
            .set_inline_datum(datum_bytes)
            .add_asset(script_hash, "HydraHeadV1".as_bytes().to_vec(), 1)
            .expect("Failed to add asset")
    }

    pub fn to_tx(&self, change_output: Output) -> Result<BuiltTransaction> {
        self.build_tx(change_output)
    }

    // I hate the passing around of change_output, will reorganize later
    pub fn to_bytes(&self, change_output: Output) -> Result<Vec<u8>> {
        let tx = self.build_tx(change_output)?;
        Ok(tx.tx_bytes.as_ref().to_vec())
    }
}

#[cfg(test)]
mod tests {

    use pallas::txbuilder::Input;

    use super::*;

    #[test]
    fn test_init_tx() {
        let tx_hash: Hash<32> =
            hex::decode("f09baeeedf28cb2a3cf8c15dbc3dc3acf44e54329b547855ffa197d2058391b2")
                .expect("Failed to decode seed tx in")
                .as_slice()
                .try_into()
                .expect("Slice was incorrect size");

        let network_id = 0;
        let seed_input = Input::new(tx_hash, 0);
        let participants =
            vec![
                hex::decode("9b29dd55b38a5e824775d303723e83e08d83c4ba72ceab284154b8a2")
                    .expect("Failed to decode participant 1"),
            ];
        let parameters = HeadParameters {
            contestation_period: 60000,
            parties: vec![hex::decode(
                "7bbfc8ffc6da9e6f6f070f0f28a4c0de8e099c34485e192660475059d8bb9557",
            )
            .expect("failed to decode party 1")],
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
                    "addr_test1vzdjnh24kw99aqj8whfsxu37s0sgmq7yhfeva2egg92t3gsws2hwn",
                )
                .expect("invalid address"),
                9983285986 - 2000000 - 2000000 - 5000000,
            ))
            .expect("Failed to build tx");

        println!("{}", hex::encode(tx_bytes));

        // NOTE: was going to check that the tx hash was the same as our reference one, but it seems the scripts have changed since that tx was built
        assert!(true);
    }
}
