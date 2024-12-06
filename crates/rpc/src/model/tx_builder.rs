use anyhow::{anyhow, bail, Context, Result};
use pallas::{
    codec::minicbor::encode,
    crypto::{hash::Hash, key::ed25519::SecretKey},
    ledger::{
        addresses::{Address, Network, ShelleyPaymentPart},
        primitives::conway::PlutusData,
        traverse::ComputeHash,
    },
    txbuilder::{BuildConway, BuiltTransaction, ExUnits, Output, ScriptKind, StagingTransaction},
};

use crate::model::{
    game::contract::{
        game_state::State,
        redeemer::{Redeemer, SpendAction},
    },
    hydra::utxo::Datum,
};

use super::{
    game::{
        contract::{game_state::GameState, validator::Validator},
        player::Player,
    },
    hydra::utxo::UTxO,
};

#[derive(Clone, Debug)]
pub struct TxBuilder {
    admin_key: SecretKey,
    pub admin_pkh: Hash<28>,
    network: Network,
}

impl TxBuilder {
    pub fn new(admin_key: SecretKey, network: Network) -> Self {
        let admin_pkh = admin_key.public_key().compute_hash();
        TxBuilder {
            admin_key,
            admin_pkh,
            network,
        }
    }

    pub fn new_game(
        &self,
        player: Player,
        utxos: Vec<UTxO>,
        player_count: u64,
        bot_count: u64,
    ) -> Result<BuiltTransaction> {
        let admin_utxos = self.find_admin_utxos(utxos);

        if admin_utxos.is_empty() {
            bail!("No admin UTxOs found");
        };

        let input_utxo = admin_utxos.first().unwrap();

        let script_address = Validator::address(self.network);
        let player_outbound_address = player
            .outbound_address(self.admin_pkh, self.network)
            .context("failed to build player multisig outbound address")?;

        let mut admin_address_bytes = self.admin_pkh.to_vec();
        admin_address_bytes.insert(
            0,
            0b1100000
                | match self.network {
                    Network::Testnet => 0,
                    Network::Mainnet => 1,
                    Network::Other(i) => i,
                },
        );
        let admin_address = Address::from_bytes(admin_address_bytes.as_slice())?;

        let game_state: PlutusData = GameState::new(self.admin_pkh.into(), player_count, bot_count)
            .add_player(player.signing_key.into())
            .into();
        let mut datum: Vec<u8> = Vec::new();
        encode(&game_state, &mut datum)?;

        let tx_builder = StagingTransaction::new()
            .input(input_utxo.clone().into())
            // GameState Datum
            .output(Output::new(script_address, 0).set_inline_datum(datum))
            // Player Output
            .output(Output::new(player_outbound_address, 0))
            //Server UTxO
            .output(Output::new(admin_address, 0))
            // Maintain Initial UTxO
            .output(Output::new(
                input_utxo.address.clone(),
                input_utxo.value.get("lovelace").unwrap_or(&0).to_owned(),
            ))
            .network_id(match self.network {
                Network::Testnet => 0,
                Network::Mainnet => 1,
                Network::Other(i) => i,
            })
            .fee(0);

        let tx = tx_builder.build_conway_raw()?;
        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;
        Ok(signed_tx)
    }

    pub fn add_player(&self, player: Player, utxos: Vec<UTxO>) -> Result<BuiltTransaction> {
        let game_state_utxo = utxos
            .clone()
            .into_iter()
            .find(|utxo| utxo.address == Validator::address(self.network))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;

        let game_state: PlutusData = GameState::try_from(game_state_utxo.datum.clone())?
            .add_player(player.signing_key.into())
            .into();

        let mut datum: Vec<u8> = Vec::new();
        encode(&game_state, &mut datum)?;

        let collateral_utxos = self.find_admin_utxos(utxos);
        let collateral_utxo = collateral_utxos
            .iter()
            .find(|utxo| utxo.value.get("lovelace").unwrap_or(&0) > &0)
            .ok_or_else(|| anyhow!("No collateral utxo found"))?;

        let script_address = Validator::address(self.network);

        let outbound_player_address = player
            .outbound_address(self.admin_pkh, self.network)
            .context("failed to construct player multisig outbound address")?;
        let redeemer: PlutusData = Redeemer::new(0, SpendAction::AddPlayer).into();
        let mut redeemer_bytes = Vec::new();
        encode(&redeemer, &mut redeemer_bytes)?;

        let tx_builder = StagingTransaction::new()
            .input(game_state_utxo.clone().into())
            .collateral_input(collateral_utxo.clone().into())
            // GameState Output
            .output(Output::new(script_address, 0).set_inline_datum(datum))
            // Player Output
            .output(Output::new(outbound_player_address, 0))
            .add_spend_redeemer(
                game_state_utxo.into(),
                redeemer_bytes,
                Some(ExUnits {
                    mem: 14000000,
                    steps: 10000000000,
                }),
            )
            .script(ScriptKind::PlutusV2, Validator::to_plutus().0.to_vec())
            .language_view(
                ScriptKind::PlutusV2,
                // These are the protocol parameters in the hydra demo devnet. They are different from the current mainnet parameters.
                vec![
                    205665, 812, 1, 1, 1000, 571, 0, 1, 1000, 24177, 4, 1, 1000, 32, 117366, 10475,
                    4, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 100,
                    100, 23000, 100, 19537, 32, 175354, 32, 46417, 4, 221973, 511, 0, 1, 89141, 32,
                    497525, 14068, 4, 2, 196500, 453240, 220, 0, 1, 1, 1000, 28662, 4, 2, 245000,
                    216773, 62, 1, 1060367, 12586, 1, 208512, 421, 1, 187000, 1000, 52998, 1,
                    80436, 32, 43249, 32, 1000, 32, 80556, 1, 57667, 4, 1000, 10, 197145, 156, 1,
                    197145, 156, 1, 204924, 473, 1, 208896, 511, 1, 52467, 32, 64832, 32, 65493,
                    32, 22558, 32, 16563, 32, 76511, 32, 196500, 453240, 220, 0, 1, 1, 69522,
                    11687, 0, 1, 60091, 32, 196500, 453240, 220, 0, 1, 1, 196500, 453240, 220, 0,
                    1, 1, 1159724, 392670, 0, 2, 806990, 30482, 4, 1927926, 82523, 4, 265318, 0, 4,
                    0, 85931, 32, 205665, 812, 1, 1, 41182, 32, 212342, 32, 31220, 32, 32696, 32,
                    43357, 32, 32247, 32, 38314, 32, 35892428, 10, 57996947, 18975, 10, 38887044,
                    32947, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
            )
            .network_id(match self.network {
                Network::Testnet => 0,
                Network::Mainnet => 1,
                Network::Other(i) => i,
            })
            .fee(0);

        let tx = tx_builder.build_conway_raw()?;
        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;

        Ok(signed_tx)
    }

    pub fn start_game(&self, utxos: Vec<UTxO>) -> Result<BuiltTransaction> {
        let game_state_utxo = utxos
            .clone()
            .into_iter()
            .find(|utxo| utxo.address == Validator::address(self.network))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;

        let game_state: PlutusData = GameState::try_from(game_state_utxo.datum.clone())?
            .set_state(State::Running)
            .into();

        let mut datum = Vec::new();
        encode(&game_state, &mut datum)?;

        let script_address = Validator::address(self.network);
        let redeemer: PlutusData = Redeemer::new(0, SpendAction::StartGame).into();
        let mut redeemer_bytes = Vec::new();
        encode(&redeemer, &mut redeemer_bytes)?;

        let collateral_utxos = self.find_admin_utxos(utxos);
        let collateral_utxo = collateral_utxos
            .iter()
            .find(|utxo| utxo.value.get("lovelace").unwrap_or(&0) > &0)
            .ok_or_else(|| anyhow!("No collateral utxo found"))?;

        let tx_builder = StagingTransaction::new()
            .input(game_state_utxo.clone().into())
            .collateral_input(collateral_utxo.clone().into())
            .output(Output::new(script_address, 0).set_inline_datum(datum))
            .add_spend_redeemer(
                game_state_utxo.into(),
                redeemer_bytes,
                Some(ExUnits {
                    mem: 14000000,
                    steps: 10000000000,
                }),
            )
            .script(ScriptKind::PlutusV2, Validator::to_plutus().0.to_vec())
            .language_view(
                ScriptKind::PlutusV2,
                // These are the protocol parameters in the hydra demo devnet. They are different from the current mainnet parameters.
                vec![
                    205665, 812, 1, 1, 1000, 571, 0, 1, 1000, 24177, 4, 1, 1000, 32, 117366, 10475,
                    4, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 100,
                    100, 23000, 100, 19537, 32, 175354, 32, 46417, 4, 221973, 511, 0, 1, 89141, 32,
                    497525, 14068, 4, 2, 196500, 453240, 220, 0, 1, 1, 1000, 28662, 4, 2, 245000,
                    216773, 62, 1, 1060367, 12586, 1, 208512, 421, 1, 187000, 1000, 52998, 1,
                    80436, 32, 43249, 32, 1000, 32, 80556, 1, 57667, 4, 1000, 10, 197145, 156, 1,
                    197145, 156, 1, 204924, 473, 1, 208896, 511, 1, 52467, 32, 64832, 32, 65493,
                    32, 22558, 32, 16563, 32, 76511, 32, 196500, 453240, 220, 0, 1, 1, 69522,
                    11687, 0, 1, 60091, 32, 196500, 453240, 220, 0, 1, 1, 196500, 453240, 220, 0,
                    1, 1, 1159724, 392670, 0, 2, 806990, 30482, 4, 1927926, 82523, 4, 265318, 0, 4,
                    0, 85931, 32, 205665, 812, 1, 1, 41182, 32, 212342, 32, 31220, 32, 32696, 32,
                    43357, 32, 32247, 32, 38314, 32, 35892428, 10, 57996947, 18975, 10, 38887044,
                    32947, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
            )
            .disclosed_signer(self.admin_pkh)
            .network_id(match self.network {
                Network::Testnet => 0,
                Network::Mainnet => 1,
                Network::Other(i) => i,
            })
            .fee(0);

        let tx = tx_builder.build_conway_raw()?;
        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;

        Ok(signed_tx)
    }

    pub fn end_game(
        &self,
        // This feels clunky, but we need to know if we are aborting the game, if they are a cheater, or a winner.
        // If this is None, we are aborting,
        // if this is Some(_, true), we are marking as cheated
        // If this is Some(_, false), we are marking as finished
        is_player_cheater: Option<(Player, bool)>,
        utxos: Vec<UTxO>,
    ) -> Result<BuiltTransaction> {
        let game_state_utxo = utxos
            .clone()
            .into_iter()
            .find(|utxo| utxo.address == Validator::address(self.network))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;

        let mut game_state: GameState = match game_state_utxo.datum.clone() {
            Datum::Hash(_) => bail!("Unexpected datum hash in game utxo"),
            Datum::Inline(data) => data.try_into()?,
            Datum::None => bail!("No datum in game utxo"),
        };

        match is_player_cheater {
            None => {
                game_state = game_state.set_state(State::Aborted);
            }
            Some((player, is_cheater)) => {
                if is_cheater {
                    game_state = game_state
                        .set_state(State::Cheated)
                        .set_cheater(player.into());
                } else {
                    game_state = game_state
                        .set_state(State::Finished)
                        .set_winner(player.into());
                };
            }
        }

        let game_state: PlutusData = game_state.into();
        let mut datum: Vec<u8> = Vec::new();
        encode(&game_state, &mut datum)?;

        let redeemer: PlutusData = Redeemer::new(0, SpendAction::EndGame).into();
        let mut redeemer_bytes: Vec<u8> = Vec::new();
        encode(&redeemer, &mut redeemer_bytes)?;

        let collateral_utxos = self.find_admin_utxos(utxos);
        let collateral_utxo = collateral_utxos
            .iter()
            .find(|utxo| utxo.value.get("lovelace").unwrap_or(&0) > &0)
            .ok_or_else(|| anyhow!("No collateral utxo found"))?;

        let tx_builder = StagingTransaction::new()
            .input(game_state_utxo.clone().into())
            .collateral_input(collateral_utxo.clone().into())
            // GameState Output
            .output(Output::new(Validator::address(self.network), 0).set_inline_datum(datum))
            .add_spend_redeemer(
                game_state_utxo.into(),
                redeemer_bytes,
                Some(ExUnits {
                    mem: 14000000,
                    steps: 10000000000,
                }),
            )
            .script(ScriptKind::PlutusV2, Validator::to_plutus().0.to_vec())
            .language_view(
                ScriptKind::PlutusV2,
                // These are the protocol parameters in the hydra demo devnet. They are different from the current mainnet parameters.
                vec![
                    205665, 812, 1, 1, 1000, 571, 0, 1, 1000, 24177, 4, 1, 1000, 32, 117366, 10475,
                    4, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 100,
                    100, 23000, 100, 19537, 32, 175354, 32, 46417, 4, 221973, 511, 0, 1, 89141, 32,
                    497525, 14068, 4, 2, 196500, 453240, 220, 0, 1, 1, 1000, 28662, 4, 2, 245000,
                    216773, 62, 1, 1060367, 12586, 1, 208512, 421, 1, 187000, 1000, 52998, 1,
                    80436, 32, 43249, 32, 1000, 32, 80556, 1, 57667, 4, 1000, 10, 197145, 156, 1,
                    197145, 156, 1, 204924, 473, 1, 208896, 511, 1, 52467, 32, 64832, 32, 65493,
                    32, 22558, 32, 16563, 32, 76511, 32, 196500, 453240, 220, 0, 1, 1, 69522,
                    11687, 0, 1, 60091, 32, 196500, 453240, 220, 0, 1, 1, 196500, 453240, 220, 0,
                    1, 1, 1159724, 392670, 0, 2, 806990, 30482, 4, 1927926, 82523, 4, 265318, 0, 4,
                    0, 85931, 32, 205665, 812, 1, 1, 41182, 32, 212342, 32, 31220, 32, 32696, 32,
                    43357, 32, 32247, 32, 38314, 32, 35892428, 10, 57996947, 18975, 10, 38887044,
                    32947, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
            )
            .disclosed_signer(self.admin_pkh)
            .network_id(match self.network {
                Network::Testnet => 0,
                Network::Mainnet => 1,
                Network::Other(i) => i,
            })
            .fee(0);

        let tx = tx_builder.build_conway_raw()?;
        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;

        Ok(signed_tx)
    }

    //TODO: sooo many clones here. Let's improve that if possible
    pub fn cleanup_game(&self, utxos: Vec<UTxO>) -> Result<BuiltTransaction> {
        let game_state_utxo = utxos
            .clone()
            .into_iter()
            .find(|utxo| utxo.address == Validator::address(self.network))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;

        let game_state: GameState = match game_state_utxo.datum.clone() {
            Datum::Hash(_) => bail!("Unexpected datum hash in game utxo"),
            Datum::Inline(data) => data
                .try_into()
                .context("failed to convert data to GameState")?,
            Datum::None => bail!("No datum in game utxo"),
        };

        let admin_utxos = self.find_admin_utxos(utxos.clone());

        let initial_state_utxo = admin_utxos
            .iter()
            .find(|utxo| utxo.value.get("lovelace").unwrap_or(&0) > &0)
            .ok_or_else(|| anyhow!("No collateral utxo found"))?;

        let redeemer: PlutusData = Redeemer::new(0, SpendAction::Collect).into();
        let mut redeemer_bytes = Vec::new();
        encode(&redeemer, &mut redeemer_bytes)?;

        let mut tx_builder = Some(
            StagingTransaction::new()
                .input(game_state_utxo.clone().into())
                .collateral_input(initial_state_utxo.clone().into())
                .output(
                    initial_state_utxo
                        .clone()
                        .try_into()
                        .context("failed to build target output from utxo object")?,
                )
                .add_spend_redeemer(
                    game_state_utxo.into(),
                    redeemer_bytes,
                    Some(ExUnits {
                        mem: 14000000,
                        steps: 10000000000,
                    }),
                )
                .script(ScriptKind::PlutusV2, Validator::to_plutus().0.to_vec())
                .language_view(
                    ScriptKind::PlutusV2,
                    // These are the protocol parameters in the hydra demo devnet. They are different from the current mainnet parameters.
                    vec![
                        205665, 812, 1, 1, 1000, 571, 0, 1, 1000, 24177, 4, 1, 1000, 32, 117366,
                        10475, 4, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 23000, 100,
                        23000, 100, 100, 100, 23000, 100, 19537, 32, 175354, 32, 46417, 4, 221973,
                        511, 0, 1, 89141, 32, 497525, 14068, 4, 2, 196500, 453240, 220, 0, 1, 1,
                        1000, 28662, 4, 2, 245000, 216773, 62, 1, 1060367, 12586, 1, 208512, 421,
                        1, 187000, 1000, 52998, 1, 80436, 32, 43249, 32, 1000, 32, 80556, 1, 57667,
                        4, 1000, 10, 197145, 156, 1, 197145, 156, 1, 204924, 473, 1, 208896, 511,
                        1, 52467, 32, 64832, 32, 65493, 32, 22558, 32, 16563, 32, 76511, 32,
                        196500, 453240, 220, 0, 1, 1, 69522, 11687, 0, 1, 60091, 32, 196500,
                        453240, 220, 0, 1, 1, 196500, 453240, 220, 0, 1, 1, 1159724, 392670, 0, 2,
                        806990, 30482, 4, 1927926, 82523, 4, 265318, 0, 4, 0, 85931, 32, 205665,
                        812, 1, 1, 41182, 32, 212342, 32, 31220, 32, 32696, 32, 43357, 32, 32247,
                        32, 38314, 32, 35892428, 10, 57996947, 18975, 10, 38887044, 32947, 10, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0,
                    ],
                )
                .network_id(match self.network {
                    Network::Testnet => 0,
                    Network::Mainnet => 1,
                    Network::Other(i) => i,
                })
                .fee(0),
        );

        // Cleanup the player state utxos
        for player in game_state.players {
            let player: Player = player.into();
            let outbound_address = player
                .outbound_address(self.admin_pkh, self.network)
                .context("failed to get player outbound address")?;
            let outbound_script = player.outbound_script(self.admin_pkh);
            let mut outbound_bytes = Vec::new();
            encode(&outbound_script, &mut outbound_bytes)
                .context("Failed to cbor encode outbound script")?;
            let player_utxos: Vec<_> = utxos
                .clone()
                .into_iter()
                .filter(|utxo| utxo.address == outbound_address)
                .collect();
            for utxo in player_utxos {
                if let Some(builder) = tx_builder {
                    tx_builder = Some(
                        builder
                            .input(utxo.clone().into())
                            .script(ScriptKind::Native, outbound_bytes.clone()),
                    )
                }
            }
        }

        // clean up any extraneous admin utxos
        for utxo in admin_utxos {
            if let Some(builder) = tx_builder {
                tx_builder = Some(builder.input(utxo.into()));
            }
        }

        let tx = tx_builder
            .ok_or(anyhow!("fatal error: no tx builder"))
            .and_then(|builder| {
                builder
                    .disclosed_signer(self.admin_pkh)
                    .build_conway_raw()
                    .map_err(|e| anyhow!("{}", e))
            })
            .map_err(|e| anyhow!("Failed to build tx: {}", e))?;

        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;

        Ok(signed_tx)
    }

    fn find_admin_utxos(&self, utxos: Vec<UTxO>) -> Vec<UTxO> {
        let admin_key = self.admin_key.public_key();
        let admin_kh = admin_key.compute_hash();

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

#[cfg(test)]
mod tests {
    use tracing::debug;

    use super::*;
    use crate::model::cluster::KeyEnvelope;
    use std::{collections::HashMap, fs::File};

    // TODO write an actual test with an assertion
    // I did this just to confirm the transaction is built as I expected manually
    #[test]
    fn test_build_new_game() {
        let admin_key: KeyEnvelope = serde_json::from_reader(
            File::open("keys/preprod.sk").expect("Failed to open key file"),
        )
        .expect("unable to parse key file");
        let tx_builder = TxBuilder::new(
            admin_key.try_into().expect("Failed to create SecretKey"),
            Network::Testnet,
        );

        let player = match Address::from_bech32(
            "addr_test1qpq0htjtaygzwtj3h4akj2mvzaxgpru4yje4ca9a507jtdw5pcy8kzccynfps4ayhmtc38j6tyjrkyfccdytnxwnd6psfelznq",
        )
        .expect("Failed to decode player address")
        {
            Address::Shelley(shelley) => shelley.payment().as_hash().clone(),
            _ => panic!("Expected Shelley address"),
        };

        let mut value: HashMap<String, u64> = HashMap::new();
        value.insert("lovelace".to_string(), 0);

        let utxos = vec![UTxO {
            hash: hex::decode("6809163f29212d08b80d619c29f0a99306ffa6e875c62121bc2b0a58da826490")
                .expect("Failed to decode hash"),
            index: 0,
            address: Address::from_bech32(
                "addr_test1vzdjnh24kw99aqj8whfsxu37s0sgmq7yhfeva2egg92t3gsws2hwn",
            )
            .expect("Failed to decode admin address"),
            datum: Datum::None,
            reference_script: None,
            value,
        }];

        let tx = tx_builder
            .new_game(player.into(), utxos, 1, 3)
            .expect("Failed to build tx");

        debug!("{}", hex::encode(tx.tx_bytes));
    }
}
