use std::ops::Deref;

use anyhow::{Context, Result};
use pallas::{
    ledger::traverse::ComputeHash,
    txbuilder::{ScriptKind, StagingTransaction},
};

use crate::model::hydra::{
    contract::head_tokens::make_head_token_script, tx::head_parameters::HeadParameters,
};

use super::input::InputWrapper;

struct InitTx {
    network_id: u8,
    seed_input: InputWrapper,
    participants: Vec<u8>,
    paramters: HeadParameters,
}

impl InitTx {
    fn build_tx(self) -> Result<StagingTransaction> {
        let script =
            make_head_token_script(&self.seed_input).context("Failed to make head token script")?;

        let tx_builder = StagingTransaction::new()
            .input(self.seed_input.into())
            .mint_asset(script.compute_hash(), "HydraHeadV1".as_bytes().to_vec(), 1)
            .context("Failed to add hydra token mint")?
            .script(ScriptKind::PlutusV2, script.as_ref().to_vec());

        todo!()
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}
