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
// Request a new game from the backend server and assign the player a hydra head with fewer players, submit tx to create initial game state, and tell them the IP address of that server
// Server will listen to all hydra heads using hydra API, detect when a player has stopped playing based on some threshold, and deallocate that user
// Eventually, also dynamically scale ec2 instances (queue players and scale up and scale down)
