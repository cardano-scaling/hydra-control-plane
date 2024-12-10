use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NewGameLocalResponse {
    pub player_state: Option<String>,
    pub admin_pkh: String,
    pub game_tx_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddPlayerLocalResponse {
    pub player_state: String,
    pub admin_pkh: String,
}
