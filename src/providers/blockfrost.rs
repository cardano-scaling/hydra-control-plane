use anyhow::{anyhow, Result};
use blockfrost::BlockfrostAPI;
use pallas::{
    crypto::hash::Hash,
    ledger::addresses::Address,
    txbuilder::{BuiltTransaction, Output},
};

pub struct Blockfrost {
    api: BlockfrostAPI,
}

impl Blockfrost {
    pub fn new(project_id: &str) -> Self {
        Self {
            api: BlockfrostAPI::new(project_id, Default::default()),
        }
    }

    pub async fn submit_transaction(&self, tx: BuiltTransaction) -> Result<String> {
        self.api
            .transactions_submit(tx.tx_bytes.0)
            .await
            .map_err(|e| anyhow!(e))
    }

    pub async fn get_utxo(&self, tx_id: &str, index: i32) -> Result<Output> {
        let transaction = self.api.transactions_utxos(tx_id).await?;

        let utxo = transaction
            .outputs
            .iter()
            .find(|output| output.output_index == index)
            .ok_or_else(|| anyhow!("Could not find output"))?;
        // Doing this conversion here because it seems there is no exported type for the utxo from blockfrost...?
        let mut output = Output::new(
            Address::from_bech32(&utxo.address.as_str())?,
            utxo.amount
                .iter()
                .find(|amount| amount.unit == "lovelace".to_string())
                .map(|amount| amount.quantity.parse::<u64>().map_err(|e| anyhow!(e)))
                .ok_or_else(|| anyhow!("failed to find lovelace"))??,
        );

        for asset in utxo.amount.iter() {
            if asset.unit != "lovelace".to_string() {
                let (policy, asset_name) = asset.unit.as_bytes().split_at(28);
                output = output.add_asset(
                    Hash::from(policy),
                    asset_name.to_vec(),
                    asset.quantity.parse::<u64>().map_err(|e| anyhow!(e))?,
                )?;
            }
        }

        if let Some(datum_hash) = &utxo.data_hash {
            output = output.set_datum_hash(Hash::from(datum_hash.as_bytes()))
        }
        if let Some(datum) = &utxo.inline_datum {
            output = output.set_inline_datum(datum.as_bytes().to_vec())
        }

        Ok(output)
    }
}
