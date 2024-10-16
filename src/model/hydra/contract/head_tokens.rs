use anyhow::{anyhow, Context};
use pallas::{
    codec::minicbor::encode,
    ledger::primitives::conway::{PlutusData, PlutusV2Script},
};
use uplc::tx::apply_params_to_script;

use crate::model::hydra::tx::input::InputWrapper;

use super::hydra_validator::HydraValidator;

pub fn make_head_token_script(input: &InputWrapper) -> anyhow::Result<PlutusV2Script> {
    let script_bytes: Vec<u8> = HydraValidator::MHead.into();
    let parameters = PlutusData::Array(vec![input.into()]);
    let mut parameter_bytes: Vec<u8> = Vec::new();
    encode(&parameters, &mut parameter_bytes).context("failed to encode parameters")?;

    let script_bytes = apply_params_to_script(parameter_bytes.as_slice(), script_bytes.as_slice())
        .map_err(|e| anyhow!("Failed to apply params to script: {}", e))?;

    Ok(PlutusV2Script(script_bytes.into()))
}
