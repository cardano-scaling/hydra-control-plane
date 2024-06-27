use super::{game_state::GameState, hydra::utxo::UTxO};

#[derive(Clone)]
pub struct Player {
    pub pkh: Vec<u8>,
    pub utxo: Option<UTxO>,
    pub utxo_time: u64,
}

impl Player {
    pub fn new(pkh: &str) -> Self {
        Player {
            pkh: pkh.as_bytes().to_owned(),
            utxo: None,
            utxo_time: 0,
        }
    }

    pub fn initialize_state(&self) -> GameState {
        GameState::new(self.pkh.clone())
    }
}
