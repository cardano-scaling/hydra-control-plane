#[derive(Clone)]
pub struct Player {
    pub pkh: String,
    pub utxo: String,
    pub utxo_time: u64,
}

impl Player {
    pub fn new(pkh: &str) -> Self {
        Player {
            pkh: pkh.to_owned(),
            utxo: String::new(),
            utxo_time: 0,
        }
    }
}
