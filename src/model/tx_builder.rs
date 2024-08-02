use anyhow::{bail, Result};
use pallas::{
    codec::minicbor::encode,
    crypto::{hash::Hash, key::ed25519::SecretKey},
    ledger::{
        addresses::{Address, ShelleyPaymentPart},
        primitives::conway::{Constr, PlutusData},
        traverse::ComputeHash,
    },
    txbuilder::{BuildBabbage, BuiltTransaction, ExUnits, Output, ScriptKind, StagingTransaction},
};

use crate::{SCRIPT_ADDRESS, SCRIPT_CBOR};

use super::{hydra::utxo::UTxO, player::Player};

#[derive(Clone)]
pub struct TxBuilder {
    admin_key: SecretKey,
    pub admin_pkh: Hash<28>,
    pub script_ref: Option<UTxO>,
}

impl TxBuilder {
    pub fn new(admin_key: SecretKey) -> Self {
        let admin_pkh = admin_key.public_key().compute_hash();
        println!("Admin PKH: {:?}", admin_pkh);
        TxBuilder {
            admin_key,
            admin_pkh,
            script_ref: None,
        }
    }

    pub fn set_script_ref(&mut self, script_ref: &UTxO) {
        self.script_ref = Some(script_ref.clone());
    }

    pub fn build_new_game_state(
        &self,
        player: &Player,
        utxos: Vec<UTxO>,
        expired_utxos: Vec<UTxO>,
    ) -> Result<BuiltTransaction> {
        if let Some(_) = player.utxo {
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
        let _ = encode(&game_state, &mut datum)?;

        let mut tx_builder = StagingTransaction::new()
            .input(input_utxo.clone().into())
            .output(Output::new(script_address, 0).set_inline_datum(datum))
            // This is so the player has collateral, we can't clean this up unfortunately
            .output(Output::new(player.address.clone(), 0))
            .output(Output::new(
                input_utxo.address.clone(),
                input_utxo.value.get("lovelace").unwrap().to_owned(),
            ))
            .change_address(input_utxo.clone().address)
            .fee(0);

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
        let tx = tx_builder.build_babbage_raw()?;
        tx.sign(self.admin_key.clone().into()).map_err(|e| e.into())
    }

    pub fn find_script_ref(utxos: Vec<UTxO>) -> Option<UTxO> {
        utxos.into_iter().find(|utxo| {
            utxo.reference_script.is_some() && utxo.address.to_bech32().unwrap() == SCRIPT_ADDRESS
        })
    }

    pub fn create_script_ref(&self, utxos: Vec<UTxO>) -> Result<BuiltTransaction> {
        let admin_utxos = self.find_admin_utxos(utxos);
        if admin_utxos.is_empty() {
            bail!("No admin UTxOs found");
        };

        let input_utxo = admin_utxos.first().unwrap();

        let script_address = Address::from_bech32(SCRIPT_ADDRESS).unwrap();

        let bytes = hex::decode(SCRIPT_CBOR).unwrap();

        let tx = StagingTransaction::new()
            .input(input_utxo.clone().into())
            .output(Output::new(script_address, 0).set_inline_script(ScriptKind::PlutusV2, bytes))
            .output(Output::new(
                input_utxo.clone().address,
                input_utxo.value.get("lovelace").unwrap().to_owned(),
            ))
            .fee(0)
            .build_babbage_raw()?;

        tx.sign(self.admin_key.clone().into()).map_err(|e| e.into())
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

    fn build_redeemer() -> Vec<u8> {
        let mut datum: Vec<u8> = Vec::new();
        let redeemer = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: Some(0),
            fields: vec![],
        });
        let _ = encode(&redeemer, &mut datum).expect("Fatal error, this should never happen");

        datum
    }
}
