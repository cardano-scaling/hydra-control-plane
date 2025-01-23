use anyhow::anyhow;
use itertools::Itertools;
use pallas::{crypto::hash::Hash, ledger::traverse::OutputRef};
use rocket::{get, State};
use rocket_errors::anyhow::Result;

use crate::LocalState;

#[get("/game/new_series?<utxo>")]
pub async fn new_series(utxo: &str, state: &State<LocalState>) -> Result<()> {
    let series_exists = {
        state
            .series_utxo
            .read()
            .map_err(|_| anyhow!("Failed to read state"))?
            .is_some()
    };

    if series_exists {
        return Result::Err(anyhow!("Series already exists").into());
    } else {
        let parts: Vec<&str> = utxo.split("#").collect_vec();
        let hash: Hash<32> = hex::decode(parts[0])
            .map_err(|_| anyhow!("invalid utxo ref"))?
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("failed to construct hash"))?;
        let ix = parts[1]
            .parse::<u64>()
            .map_err(|_| anyhow!("invalid utxo ref"))?;
        let output_ref = OutputRef::new(hash, ix);

        state
            .series_utxo
            .write()
            .map_err(|_| anyhow!("Failed to write state"))?
            .clone_from(&Some(output_ref));

        Ok(())
    }
}
