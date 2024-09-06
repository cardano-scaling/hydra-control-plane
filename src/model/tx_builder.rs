use anyhow::{bail, Context, Result};
use pallas::{
    codec::minicbor::encode,
    crypto::{hash::Hash, key::ed25519::SecretKey},
    ledger::{
        addresses::{Address, ShelleyPaymentPart},
        primitives::conway::{Constr, PlutusData},
        traverse::ComputeHash,
    },
    txbuilder::{BuildBabbage, BuiltTransaction, Output, StagingTransaction},
};

use super::{hydra::utxo::UTxO, player::Player};
use crate::SCRIPT_ADDRESS;

#[derive(Clone)]
pub struct TxBuilder {
    admin_key: SecretKey,
    pub admin_pkh: Hash<28>,
}

impl TxBuilder {
    pub fn new(admin_key: SecretKey) -> Self {
        let admin_pkh = admin_key.public_key().compute_hash();
        TxBuilder {
            admin_key,
            admin_pkh,
        }
    }

    pub fn build_new_game_state(
        &self,
        player: &Player,
        utxos: Vec<UTxO>,
        _expired_utxos: Vec<UTxO>,
        collateral_addr: Address,
    ) -> Result<(BuiltTransaction, Vec<u8>)> {
        if player.utxo.is_some() {
            bail!("Player already has a UTxO created");
        }

        let admin_utxos = self.find_admin_utxos(utxos);

        if admin_utxos.is_empty() {
            bail!("No admin UTxOs found");
        };

        let input_utxo = admin_utxos.first().unwrap();

        let script_address = Address::from_bech32(SCRIPT_ADDRESS).unwrap();

        let game_state: PlutusData = player
            .initialize_state(self.admin_pkh.as_ref().to_vec())
            .into();
        let mut datum: Vec<u8> = Vec::new();
        encode(&game_state, &mut datum)?;

        let tx_builder = StagingTransaction::new()
            .input(input_utxo.clone().into())
            .output(Output::new(script_address, 0).set_inline_datum(datum.clone()))
            // This is so the player has collateral, we can't clean this up unfortunately
            .output(Output::new(collateral_addr, 0))
            .output(Output::new(
                input_utxo.address.clone(),
                input_utxo.value.get("lovelace").unwrap().to_owned(),
            ))
            .change_address(input_utxo.clone().address)
            .fee(0);

        /* Skipping cleanup for now
        if expired_utxos.len() > 0 {
            tx_builder = tx_builder
                .reference_input(
                    self.script_ref
                        .as_ref()
                        .expect("must have script ref by this point")
                        .clone()
                        .into(),
                )
                .collateral_input(input_utxo.clone().into());
        }
        for utxo in expired_utxos {
            tx_builder = tx_builder.input(utxo.clone().into());
            tx_builder = tx_builder.add_spend_redeemer(
                utxo.into(),
                TxBuilder::build_redeemer(),
                Some(ExUnits {
                    mem: 7000000,
                    steps: 3000000000,
                }),
            );
        }
        */
        let tx = tx_builder.build_babbage_raw()?;
        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;
        Ok((signed_tx, datum))
    }

    fn find_admin_utxos(&self, utxos: Vec<UTxO>) -> Vec<UTxO> {
        let admin_kh = self.admin_key.public_key().compute_hash();
        utxos
            .into_iter()
            .filter(|utxo| match &utxo.address {
                Address::Shelley(address) => match address.payment() {
                    ShelleyPaymentPart::Key(hash) => hash == &admin_kh,
                    ShelleyPaymentPart::Script(hash) => hash == &admin_kh,
                },
                _ => false,
            })
            .collect()
    }

    #[allow(dead_code)]
    fn build_redeemer() -> Vec<u8> {
        let mut datum: Vec<u8> = Vec::new();
        let redeemer = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: Some(0),
            fields: vec![],
        });
        encode(&redeemer, &mut datum).expect("Fatal error, this should never happen");

        datum
    }
}
