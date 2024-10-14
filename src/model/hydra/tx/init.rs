use anyhow::{anyhow, Context, Result};
use pallas::{
    codec::minicbor::encode,
    crypto::hash::Hash,
    ledger::{addresses::Address, traverse::ComputeHash},
    txbuilder::{BuildBabbage, BuiltTransaction, Output, ScriptKind, StagingTransaction},
};

use crate::model::hydra::{
    contract::{head_tokens::make_head_token_script, head_validator::head_validator},
    tx::head_parameters::HeadParameters,
};

use super::input::InputWrapper;

struct InitTx {
    network_id: u8,
    seed_input: InputWrapper,
    participants: Vec<u8>,
    paramters: HeadParameters,
}

impl InitTx {
    fn build_tx(&self) -> Result<BuiltTransaction> {
        let script =
            make_head_token_script(&self.seed_input).context("Failed to make head token script")?;

        let script_hash = script.compute_hash();
        let tx_builder = StagingTransaction::new()
            .network_id(self.network_id)
            .input(self.seed_input.clone().into())
            .mint_asset(script_hash, "HydraHeadV1".as_bytes().to_vec(), 1)
            .context("Failed to add hydra token mint")?
            .script(ScriptKind::PlutusV2, script.as_ref().to_vec())
            .output(self.make_head_output_initial(script_hash));

        tx_builder
            .build_babbage_raw()
            .map_err(|e| anyhow!("Failed to build tx: {}", e))
    }

    // TODO: actually do proper error handling here
    fn make_head_output_initial(&self, script_hash: Hash<28>) -> Output {
        let datum = self.paramters.to_head_datum(script_hash, &self.seed_input);
        let mut datum_bytes = Vec::new();
        encode(&datum, &mut datum_bytes).expect("failed to encode parameters");
        let hydra_script = head_validator();
        let mut address_bytes = hydra_script.compute_hash().to_vec();
        address_bytes.insert(0, 0b01110000 | self.network_id);
        let address =
            Address::from_bytes(address_bytes.as_slice()).expect("Failed to create address");
        let output = Output::new(address, 1600000)
            .set_inline_datum(datum_bytes)
            .add_asset(script_hash, "HydraHeadV1".as_bytes().to_vec(), 1)
            .expect("Failed to add asset");

        output
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}
