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

    pub fn initialize_state(&self) -> GameState {
        GameState::new(self.pkh.clone())
    }

    pub fn generate_state_update(&mut self, byte_count: u64, new_state: GameState) -> StateUpdate {
        let state_update = if let Some(old_state) = &self.game_state {
            StateUpdate {
                bytes: byte_count,
                kills: new_state.player.kill_count - old_state.player.kill_count,
                items: 0,
                secrets: 0,
                play_time: 0,
            }
        } else {
            StateUpdate {
                bytes: byte_count,
                kills: new_state.player.kill_count,
                items: 0,
                secrets: 0,
                play_time: 0,
            }
        };

        self.game_state = Some(new_state);

        state_update
    }
}
