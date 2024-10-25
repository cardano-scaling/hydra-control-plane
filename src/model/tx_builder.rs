use anyhow::{bail, Context, Result};
use pallas::{
    codec::{minicbor::encode, utils::MaybeIndefArray},
    crypto::{hash::Hash, key::ed25519::SecretKey},
    ledger::{
        addresses::{Address, Network, PaymentKeyHash, ShelleyPaymentPart},
        primitives::conway::{Constr, PlutusData},
        traverse::ComputeHash,
    },
    txbuilder::{BuildConway, BuiltTransaction, Output, StagingTransaction},
};

use crate::SCRIPT_ADDRESS;

use super::{datums::game_state::GameState, hydra::utxo::UTxO};

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

    pub fn build_new_game(
        &self,
        player: PaymentKeyHash,
        utxos: Vec<UTxO>,
        network: Network,
    ) -> Result<BuiltTransaction> {
        let admin_utxos = self.find_admin_utxos(utxos);

        if admin_utxos.is_empty() {
            bail!("No admin UTxOs found");
        };

        let input_utxo = admin_utxos.first().unwrap();

        let script_address = Address::from_bech32(SCRIPT_ADDRESS).unwrap();
        let mut player_address_bytes = player.to_vec();
        player_address_bytes.insert(0, 0b01100000 | network.into());
        let player_address = Address::from_bytes(player_address_bytes.as_slice()).unwrap();

        let game_state: PlutusData = GameState::new(self.admin_pkh.into())
            .add_player(player.into())
            .into();
        let mut datum: Vec<u8> = Vec::new();
        encode(&game_state, &mut datum)?;

        let tx_builder = StagingTransaction::new()
            .input(input_utxo.clone().into())
            // GameState Datum
            .output(Output::new(script_address, 0).set_inline_datum(datum.clone()))
            // Player Output
            .output(Output::new(player_address, 0))
            // Maintain Initial UTxO
            .output(Output::new(
                input_utxo.address.clone(),
                input_utxo.value.get("lovelace").unwrap().to_owned(),
            ))
            .fee(0);

        let tx = tx_builder.build_conway_raw()?;
        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;
        Ok(signed_tx)
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
            fields: MaybeIndefArray::Indef(vec![]),
        });
        encode(&redeemer, &mut datum).expect("Fatal error, this should never happen");

        datum
    }
}
