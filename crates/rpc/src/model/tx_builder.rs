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
        player: Player,
        utxos: Vec<UTxO>,
        network: Network,
    ) -> Result<BuiltTransaction> {
        let admin_utxos = self.find_admin_utxos(utxos);

        if admin_utxos.is_empty() {
            bail!("No admin UTxOs found");
        };

        let input_utxo = admin_utxos.first().unwrap();

        let script_address = Validator::address(network);
        println!("Script Address: {:?}", script_address.to_bech32());
        let player_inbound_address = player
            .inbound_address(self.admin_pkh, network)
            .context("failed to build player multisig inbound address")?;
        let player_outbound_address = player
            .outbound_address(self.admin_pkh, network)
            .context("failed to build player multisig outbound address")?;

        let game_state: PlutusData = GameState::new(self.admin_pkh.into())
            .add_player(player.signing_key.into())
            .into();
        let mut datum: Vec<u8> = Vec::new();
        encode(&game_state, &mut datum)?;

        let tx_builder = StagingTransaction::new()
            .input(input_utxo.clone().into())
            // GameState Datum
            .output(Output::new(script_address, 0).set_inline_datum(datum))
            // Player Output
            .output(Output::new(player_inbound_address, 0))
            .output(Output::new(player_outbound_address, 0))
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

    pub fn add_player(
        &self,
        player: Player,
        utxos: Vec<UTxO>,
        network: Network,
    ) -> Result<BuiltTransaction> {
        let game_state_utxo = utxos
            .clone()
            .into_iter()
            .find(|utxo| utxo.address == Validator::address(network))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;

        let game_state: PlutusData = match game_state_utxo.datum.clone() {
            Datum::Hash(_) => bail!("Unexpected datum hash in game utxo"),
            Datum::Inline(data) => {
                GameState::try_from(data).context("failed to decode plutus data for game state")?
            }
            Datum::None => bail!("No datum in game utxo"),
        }
        .add_player(player.signing_key.into())
        .into();

        let mut datum: Vec<u8> = Vec::new();
        encode(&game_state, &mut datum)?;

        let collateral_utxos = self.find_admin_utxos(utxos);
        let collateral_utxo = collateral_utxos
            .get(0)
            .ok_or_else(|| anyhow!("No collateral utxo found"))?;

        let script_address = Validator::address(network);

        let inbound_player_address = player
            .inbound_address(self.admin_pkh, network)
            .context("failed to construct player multisig inbound address")?;
        let outbound_player_address = player
            .outbound_address(self.admin_pkh, network)
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
            .output(Output::new(inbound_player_address, 0))
            .output(Output::new(outbound_player_address, 0))
            .add_spend_redeemer(
                game_state_utxo.into(),
                redeemer_bytes,
                Some(ExUnits {
                    mem: 14000000,
                    steps: 10000000000,
                }),
            )
            .script(ScriptKind::PlutusV3, Validator::to_plutus().0.to_vec())
            .language_view(
                ScriptKind::PlutusV3,
                // These are the protocol parameters in the hydra demo devnet. They are different from the current mainnet parameters.
                vec![
                    100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356,
                    4, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100,
                    100, 16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32,
                    91189, 769, 4, 2, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 1,
                    1000, 42921, 4, 2, 24548, 29498, 38, 1, 898148, 27279, 1, 51775, 558, 1, 39184,
                    1000, 60594, 1, 141895, 32, 83150, 32, 15299, 32, 76049, 1, 13169, 4, 22100,
                    10, 28999, 74, 1, 28999, 74, 1, 43285, 552, 1, 44749, 541, 1, 33852, 32, 68246,
                    32, 72362, 32, 7243, 32, 7391, 32, 11546, 32, 85848, 123203, 7305, -900, 1716,
                    549, 57, 85848, 0, 1, 90434, 519, 0, 1, 74433, 32, 85848, 123203, 7305, -900,
                    1716, 549, 57, 85848, 0, 1, 1, 85848, 123203, 7305, -900, 1716, 549, 57, 85848,
                    0, 1, 955506, 213312, 0, 2, 270652, 22588, 4, 1457325, 64566, 4, 20467, 1, 4,
                    0, 141992, 32, 100788, 420, 1, 1, 81663, 32, 59498, 32, 20142, 32, 24588, 32,
                    20744, 32, 25933, 32, 24623, 32, 43053543, 10, 53384111, 14333, 10, 43574283,
                    26308, 10, 16000, 100, 16000, 100, 962335, 18, 2780678, 6, 442008, 1, 52538055,
                    3756, 18, 267929, 18, 76433006, 8868, 18, 52948122, 18, 1995836, 36, 3227919,
                    12, 901022, 1, 166917843, 4307, 36, 284546, 36, 158221314, 26549, 36, 74698472,
                    36, 333849714, 1, 254006273, 72, 2174038, 72, 2261318, 64571, 4, 207616, 8310,
                    4, 1293828, 28716, 63, 0, 1, 1006041, 43623, 251, 0, 1,
                ],
            )
            .fee(0);

        let tx = tx_builder.build_conway_raw()?;
        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;

        Ok(signed_tx)
    }

    pub fn end_game(
        &self,
        player: Player,
        is_cheater: bool,
        utxos: Vec<UTxO>,
        network: Network,
    ) -> Result<BuiltTransaction> {
        let game_state_utxo = utxos
            .clone()
            .into_iter()
            .find(|utxo| utxo.address == Validator::address(network))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;

        let mut game_state: GameState = match game_state_utxo.datum.clone() {
            Datum::Hash(_) => bail!("Unexpected datum hash in game utxo"),
            Datum::Inline(data) => data.try_into()?,
            Datum::None => bail!("No datum in game utxo"),
        };
        if is_cheater {
            game_state = game_state
                .set_state(State::CHEATED)
                .set_cheater(player.into());
        } else {
            game_state = game_state
                .set_state(State::FINISHED)
                .set_winner(player.into());
        };

        let game_state: PlutusData = game_state.into();
        let mut datum: Vec<u8> = Vec::new();
        encode(&game_state, &mut datum)?;

        let redeemer: PlutusData = Redeemer::new(0, SpendAction::EndGame).into();
        let mut redeemer_bytes: Vec<u8> = Vec::new();
        encode(&redeemer, &mut redeemer_bytes)?;

        let collateral_utxos = self.find_admin_utxos(utxos);
        let collateral_utxo = collateral_utxos
            .get(0)
            .ok_or_else(|| anyhow!("No collateral utxo found"))?;

        let tx_builder = StagingTransaction::new()
            .input(game_state_utxo.clone().into())
            .collateral_input(collateral_utxo.clone().into())
            // GameState Output
            .output(Output::new(Validator::address(network), 0).set_inline_datum(datum))
            .add_spend_redeemer(
                game_state_utxo.into(),
                redeemer_bytes,
                Some(ExUnits {
                    mem: 14000000,
                    steps: 10000000000,
                }),
            )
            .script(ScriptKind::PlutusV3, Validator::to_plutus().0.to_vec())
            .language_view(
                ScriptKind::PlutusV3,
                // These are the protocol parameters in the hydra demo devnet. They are different from the current mainnet parameters.
                vec![
                    100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356,
                    4, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100,
                    100, 16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32,
                    91189, 769, 4, 2, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 1,
                    1000, 42921, 4, 2, 24548, 29498, 38, 1, 898148, 27279, 1, 51775, 558, 1, 39184,
                    1000, 60594, 1, 141895, 32, 83150, 32, 15299, 32, 76049, 1, 13169, 4, 22100,
                    10, 28999, 74, 1, 28999, 74, 1, 43285, 552, 1, 44749, 541, 1, 33852, 32, 68246,
                    32, 72362, 32, 7243, 32, 7391, 32, 11546, 32, 85848, 123203, 7305, -900, 1716,
                    549, 57, 85848, 0, 1, 90434, 519, 0, 1, 74433, 32, 85848, 123203, 7305, -900,
                    1716, 549, 57, 85848, 0, 1, 1, 85848, 123203, 7305, -900, 1716, 549, 57, 85848,
                    0, 1, 955506, 213312, 0, 2, 270652, 22588, 4, 1457325, 64566, 4, 20467, 1, 4,
                    0, 141992, 32, 100788, 420, 1, 1, 81663, 32, 59498, 32, 20142, 32, 24588, 32,
                    20744, 32, 25933, 32, 24623, 32, 43053543, 10, 53384111, 14333, 10, 43574283,
                    26308, 10, 16000, 100, 16000, 100, 962335, 18, 2780678, 6, 442008, 1, 52538055,
                    3756, 18, 267929, 18, 76433006, 8868, 18, 52948122, 18, 1995836, 36, 3227919,
                    12, 901022, 1, 166917843, 4307, 36, 284546, 36, 158221314, 26549, 36, 74698472,
                    36, 333849714, 1, 254006273, 72, 2174038, 72, 2261318, 64571, 4, 207616, 8310,
                    4, 1293828, 28716, 63, 0, 1, 1006041, 43623, 251, 0, 1,
                ],
            )
            .disclosed_signer(self.admin_pkh)
            .fee(0);

        let tx = tx_builder.build_conway_raw()?;
        let signed_tx = tx
            .sign(self.admin_key.clone().into())
            .context("failed to sign tx")?;

        Ok(signed_tx)
    }

    //TODO: sooo many clones here. Let's improve that if possible
    pub fn cleanup_game(&self, utxos: Vec<UTxO>, network: Network) -> Result<BuiltTransaction> {
        let game_state_utxo = utxos
            .clone()
            .into_iter()
            .find(|utxo| utxo.address == Validator::address(network))
            .ok_or_else(|| anyhow!("game state UTxO not found"))?;

        let game_state: GameState = match game_state_utxo.datum.clone() {
            Datum::Hash(_) => bail!("Unexpected datum hash in game utxo"),
            Datum::Inline(data) => data
                .try_into()
                .context("failed to convert data to GameState")?,
            Datum::None => bail!("No datum in game utxo"),
        };

        let collateral_utxos = self.find_admin_utxos(utxos.clone());

        let collateral_utxo = collateral_utxos
            .get(0)
            .ok_or_else(|| anyhow!("No collateral utxo found"))?;

        let redeemer: PlutusData = Redeemer::new(0, SpendAction::Collect).into();
        let mut redeemer_bytes = Vec::new();
        encode(&redeemer, &mut redeemer_bytes)?;

        let mut tx_builder = Some(
            StagingTransaction::new()
                .input(game_state_utxo.clone().into())
                .input(collateral_utxo.clone().into())
                .collateral_input(collateral_utxo.clone().into())
                .output(
                    collateral_utxo
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
                .script(ScriptKind::PlutusV3, Validator::to_plutus().0.to_vec())
                .language_view(
                    ScriptKind::PlutusV3,
                    // These are the protocol parameters in the hydra demo devnet. They are different from the current mainnet parameters.
                    vec![
                        100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305,
                        8356, 4, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000,
                        100, 100, 100, 16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0,
                        1, 22151, 32, 91189, 769, 4, 2, 85848, 123203, 7305, -900, 1716, 549, 57,
                        85848, 0, 1, 1, 1000, 42921, 4, 2, 24548, 29498, 38, 1, 898148, 27279, 1,
                        51775, 558, 1, 39184, 1000, 60594, 1, 141895, 32, 83150, 32, 15299, 32,
                        76049, 1, 13169, 4, 22100, 10, 28999, 74, 1, 28999, 74, 1, 43285, 552, 1,
                        44749, 541, 1, 33852, 32, 68246, 32, 72362, 32, 7243, 32, 7391, 32, 11546,
                        32, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 90434, 519, 0,
                        1, 74433, 32, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 1,
                        85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 955506, 213312, 0,
                        2, 270652, 22588, 4, 1457325, 64566, 4, 20467, 1, 4, 0, 141992, 32, 100788,
                        420, 1, 1, 81663, 32, 59498, 32, 20142, 32, 24588, 32, 20744, 32, 25933,
                        32, 24623, 32, 43053543, 10, 53384111, 14333, 10, 43574283, 26308, 10,
                        16000, 100, 16000, 100, 962335, 18, 2780678, 6, 442008, 1, 52538055, 3756,
                        18, 267929, 18, 76433006, 8868, 18, 52948122, 18, 1995836, 36, 3227919, 12,
                        901022, 1, 166917843, 4307, 36, 284546, 36, 158221314, 26549, 36, 74698472,
                        36, 333849714, 1, 254006273, 72, 2174038, 72, 2261318, 64571, 4, 207616,
                        8310, 4, 1293828, 28716, 63, 0, 1, 1006041, 43623, 251, 0, 1,
                    ],
                )
                .fee(0),
        );

        for player in game_state.players {
            let player: Player = player.into();
            let inbound_address = player
                .inbound_address(self.admin_pkh, network)
                .context("failed to get player inbound address")?;
            let outbound_address = player
                .outbound_address(self.admin_pkh, network)
                .context("failed to get player outbound address")?;
            let inbound_script = player.inbound_script(self.admin_pkh);
            let mut inbound_bytes = Vec::new();
            encode(&inbound_script, &mut inbound_bytes)
                .context("Failed to cbor encode outbound script")?;
            let outbound_script = player.outbound_script(self.admin_pkh);
            let mut outbound_bytes = Vec::new();
            encode(&outbound_script, &mut outbound_bytes)
                .context("Failed to cbor encode outbound script")?;
            let player_utxos: Vec<_> = utxos
                .clone()
                .into_iter()
                .filter(|utxo| utxo.address == inbound_address || utxo.address == outbound_address)
                .collect();
            for utxo in player_utxos {
                if let Some(builder) = tx_builder {
                    tx_builder = Some(builder.input(utxo.clone().into()).script(
                        ScriptKind::Native,
                        if utxo.address == inbound_address {
                            inbound_bytes.clone()
                        } else {
                            outbound_bytes.clone()
                        },
                    ))
                }
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
        let admin_key: KeyEnvelope =
            serde_json::from_reader(File::open("preprod.sk").expect("Failed to open key file"))
                .expect("unable to parse key file");
        let tx_builder = TxBuilder::new(admin_key.try_into().expect("Failed to create SecretKey"));

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
            .build_new_game(player.into(), utxos, Network::Testnet)
            .expect("Failed to build tx");

        debug!("{}", hex::encode(tx.tx_bytes));
    }
}