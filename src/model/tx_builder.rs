use pallas::{
    codec::minicbor::encode,
    crypto::key::ed25519::SecretKey,
    ledger::{
        addresses::{Address, ShelleyPaymentPart},
        primitives::conway::PlutusData,
        traverse::ComputeHash,
    },
    txbuilder::{BuildBabbage, BuiltTransaction, Input, Output, StagingTransaction},
};

use hex::FromHex;

use crate::SCRIPT_ADDRESS;

use super::{hydra::utxo::UTxO, player::Player};

#[derive(Clone)]
pub struct TxBuilder {
    admin_key: SecretKey,
    script_ref: Option<Output>,
}

impl TxBuilder {
    pub fn new(admin_key: [u8; 32]) -> Self {
        let admin_key: SecretKey = admin_key.into();
        TxBuilder {
            admin_key,
            script_ref: None,
        }
    }

    pub fn set_script_ref(&mut self, script_ref: &UTxO) -> Result<(), Box<dyn std::error::Error>> {
        let script_ref: Output = script_ref.clone().try_into()?;
        self.script_ref = Some(script_ref);

        Ok(())
    }

    pub fn build_new_game_state(
        &self,
        player: &Player,
        utxos: Vec<UTxO>,
    ) -> Result<BuiltTransaction, Box<dyn std::error::Error>> {
        if let Some(_) = player.utxo {
            return Err("Player already has a UTxO created".into());
        }

        let admin_kh = self.admin_key.public_key().compute_hash();
        let admin_utxos: Vec<UTxO> = utxos
            .into_iter()
            .filter(|utxo| {
                println!(
                    "utxo: {:?} | address: {:?}",
                    utxo.hash,
                    utxo.address.to_bech32().unwrap()
                );
                match &utxo.address {
                    Address::Shelley(address) => match address.payment() {
                        ShelleyPaymentPart::Key(hash) => {
                            println!("{:?} | {:?}", hash, &admin_kh);
                            hash == &admin_kh
                        }
                        ShelleyPaymentPart::Script(hash) => hash == &admin_kh,
                    },
                    _ => false,
                }
            })
            .collect();

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
}
