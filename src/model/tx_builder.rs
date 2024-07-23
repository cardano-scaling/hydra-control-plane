use pallas::{
    codec::minicbor::encode,
    crypto::key::ed25519::SecretKey,
    ledger::{
        addresses::{Address, ShelleyPaymentPart},
        primitives::conway::PlutusData,
        traverse::ComputeHash,
    },
    txbuilder::{BuildBabbage, BuiltTransaction, Output, ScriptKind, StagingTransaction},
};

use crate::{SCRIPT_ADDRESS, SCRIPT_CBOR};

use super::{hydra::utxo::UTxO, player::Player};

#[derive(Clone)]
pub struct TxBuilder {
    admin_key: SecretKey,
    pub script_ref: Option<UTxO>,
}

impl TxBuilder {
    pub fn new(admin_key: SecretKey) -> Self {
        TxBuilder {
            admin_key,
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
    ) -> Result<BuiltTransaction, Box<dyn std::error::Error>> {
        if let Some(_) = player.utxo {
            return Err("Player already has a UTxO created".into());
        }

        let admin_utxos = self.find_admin_utxos(utxos);

        if admin_utxos.is_empty() {
            return Err("No admin UTxOs found".into());
        };

        let input_utxo = admin_utxos.first().unwrap();

        let script_address = Address::from_bech32(SCRIPT_ADDRESS).unwrap();

        let game_state: PlutusData = player.initialize_state().into();
        let mut datum: Vec<u8> = Vec::new();
        let _ = encode(&game_state, &mut datum)?;

        let tx = StagingTransaction::new()
            .input(input_utxo.clone().into())
            .output(Output::new(script_address, 0).set_inline_datum(datum))
            .output(Output::new(player.address.clone(), 0))
            // Temp workaround until we query the script address for the latestUTxO in browser, this way player has collateral
            .output(Output::new(player.address.clone(), 0))
            .output(Output::new(
                input_utxo.address.clone(),
                input_utxo.value.get("lovelace").unwrap().to_owned(),
            ))
            .change_address(input_utxo.clone().address)
            .fee(0)
            .build_babbage_raw()?;

        tx.sign(self.admin_key.clone().into()).map_err(|e| e.into())
    }

    pub fn find_script_ref(utxos: Vec<UTxO>) -> Option<UTxO> {
        utxos.into_iter().find(|utxo| {
            utxo.reference_script.is_some() && utxo.address.to_bech32().unwrap() == SCRIPT_ADDRESS
        })
    }

    pub fn create_script_ref(
        &self,
        utxos: Vec<UTxO>,
    ) -> Result<BuiltTransaction, Box<dyn std::error::Error>> {
        let admin_utxos = self.find_admin_utxos(utxos);
        if admin_utxos.is_empty() {
            return Err("No admin UTxOs found".into());
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
}
