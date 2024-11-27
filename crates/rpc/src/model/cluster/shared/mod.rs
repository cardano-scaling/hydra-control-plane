use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NewGameLocalResponse {
    pub player_state: String,
    pub admin_pkh: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddPlayerLocalResponse {
    pub player_state: String,
    pub admin_pkh: String,
}
