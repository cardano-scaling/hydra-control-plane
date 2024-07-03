use super::{game_state::GameState, hydra::utxo::UTxO, node::StateUpdate};

#[derive(Clone)]
pub struct Player {
    pub pkh: Vec<u8>,
    pub utxo: Option<UTxO>,
    pub utxo_time: u64,
    pub game_state: Option<GameState>,
}

impl Player {
    pub fn new(pkh: &str) -> Self {
        Player {
            pkh: pkh.as_bytes().to_owned(),
            utxo: None,
            utxo_time: 0,
            game_state: None,
        }
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
