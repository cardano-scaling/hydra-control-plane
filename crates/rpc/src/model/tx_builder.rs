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
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

use crate::model::{
    game::contract::{
        game_state::State,
        redeemer::{Redeemer, SpendAction},
    },
    hydra::utxo::Datum,
};

use super::{
    game::{
        contract::{
            game_state::{GameState, PaymentCredential},
            validator::Validator,
        },
        player::Player,
    },
    hydra::utxo::UTxO,
};

#[derive(Clone, Debug)]
pub struct TxBuilder {
    admin_key: SecretKey,
    pub admin_pkh: Hash<28>,
    pub network: Network,
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

    pub fn new_game(&self, player: Option<Player>, utxos: Vec<UTxO>) -> Result<BuiltTransaction> {
        let admin_utxos = self.find_admin_utxos(utxos);

        if admin_utxos.is_empty() {
            bail!("No admin UTxOs found");
        };

        let input_utxo = admin_utxos.first().unwrap();

        let player_outbound_address = if let Some(ref player) = player {
            Some(
                player
                    .outbound_address(self.admin_pkh, self.network)
                    .context("failed to build player multisig outbound address")?,
            )
        } else {
            None
        };

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

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let millis = now.as_millis() as u64;

        let tx_builder = StagingTransaction::new()
            .invalid_from_slot(millis + 3600 * 24 * 365)
            .input(input_utxo.clone().into());
        // Player Output
        let tx_builder = if let Some(poa) = player_outbound_address {
            tx_builder.output(Output::new(poa, 0))
        } else {
            tx_builder
        };
        let tx_builder = tx_builder
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
        let admin_utxos = self.find_admin_utxos(utxos);

        if admin_utxos.is_empty() {
            bail!("No admin UTxOs found");
        };

        let input_utxo = admin_utxos.first().unwrap();

        let outbound_player_address = player
            .outbound_address(self.admin_pkh, self.network)
            .context("failed to construct player multisig outbound address")?;
        let redeemer: PlutusData = Redeemer::new(0, SpendAction::AddPlayer).into();
        let mut redeemer_bytes = Vec::new();
        encode(&redeemer, &mut redeemer_bytes).context("failed to encode redeemer")?;
        info!("Redeemer constructed");

        let tx_builder = StagingTransaction::new()
            .input(input_utxo.clone().into())
            // Player Output
            .output(Output::new(outbound_player_address, 0))
            .network_id(match self.network {
                Network::Testnet => 0,
                Network::Mainnet => 1,
                Network::Other(i) => i,
            })
            .fee(0);

        let tx = tx_builder
            .build_conway_raw()
            .context("failed to build conway transaction")?;

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
    pub fn cleanup_game(
        &self,
        utxos: Vec<UTxO>,
        players: Vec<PaymentCredential>,
    ) -> Result<BuiltTransaction> {
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
                .collateral_input(initial_state_utxo.clone().into())
                .output(
                    initial_state_utxo
                        .clone()
                        .try_into()
                        .context("failed to build target output from utxo object")?,
                )
                .network_id(match self.network {
                    Network::Testnet => 0,
                    Network::Mainnet => 1,
                    Network::Other(i) => i,
                })
                .fee(0),
        );

        // Cleanup the player state utxos
        for player in players {
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

        println!("{}", hex::encode(admin_kh));

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
            .new_game(Some(player.into()), utxos)
            .expect("Failed to build tx");

        debug!("{}", hex::encode(tx.tx_bytes));
    }
}
