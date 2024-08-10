use std::{collections::HashMap, time::Duration};

use pallas::ledger::addresses::Address;

use super::{game_state::GameState, hydra::utxo::UTxO, node::StateUpdate};
use anyhow::{bail, Result};
#[allow(dead_code)]
#[derive(Clone)]
pub struct Player {
    pub pkh: Vec<u8>,
    pub address: Address,
    pub utxo: Option<UTxO>,
    pub utxo_time: u64,
    pub game_state: Option<GameState>,
}

impl Player {
    pub fn new(address: Address) -> Result<Self> {
        let pkh = match &address {
            Address::Shelley(shelley) => shelley.payment().to_vec(),
            _ => bail!("Invalid address type"),
        };

        Ok(Player {
            pkh,
            address,
            utxo: None,
            utxo_time: 0,
            game_state: None,
        })
    }

    pub fn initialize_state(&self, admin_pkh: Vec<u8>) -> GameState {
        GameState::new(self.pkh.clone(), admin_pkh)
    }

    pub fn generate_state_update(&mut self, byte_count: u64, new_state: GameState) -> StateUpdate {
        let mut play_time = HashMap::new();
        play_time.insert(hex::encode(&new_state.owner), new_state.leveltime.clone());
        let state_update = if let Some(old_state) = &self.game_state {
            StateUpdate {
                bytes: byte_count,
                kills: new_state.player.total_stats.kill_count
                    - old_state.player.total_stats.kill_count,
                items: new_state.player.total_stats.item_count
                    - old_state.player.total_stats.item_count,
                secrets: new_state.player.total_stats.secret_count
                    - old_state.player.total_stats.secret_count,
                play_time,
            }
        } else {
            StateUpdate {
                bytes: byte_count,
                kills: new_state.player.total_stats.kill_count,
                items: new_state.player.total_stats.item_count,
                secrets: new_state.player.total_stats.secret_count,
                play_time,
            }
        };

        self.game_state = Some(new_state);

        state_update
    }

    pub fn is_expired(&self, duration: Duration) -> bool {
        if let None = self.utxo {
            // if we don't have a utxo yet, we haven't started playing
            return false;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards...")
            .as_secs();

        now - self.utxo_time > duration.as_secs()
    }
}
