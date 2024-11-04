use std::path::PathBuf;

use ::serde::Deserialize;
use pallas::crypto::key::ed25519::SecretKey;

pub mod model;
pub mod providers;

#[derive(Debug, Deserialize)]
pub struct NodeConfig {
    #[serde(default = "localhost")]
    pub local_url: String,
    pub remote_url: Option<String>,
    #[serde(default = "default_region")]
    pub region: String,
    pub port: u32,

    pub max_players: usize,
    pub admin_key_file: PathBuf,
    pub persisted: bool,
    pub reserved: bool,
}

fn default_region() -> String {
    "us-east-2".to_string()
}

fn localhost() -> String {
    "ws://127.0.0.1".to_string()
}

pub static TEMP_ADMIN_KEY: std::sync::LazyLock<SecretKey> = std::sync::LazyLock::new(|| {
    let bytes: [u8; 32] =
        hex::decode("AEA9DC3E07D9926AFC62F537636DF700D216B98E7217B83B2C1098E31DAF0D6F")
            .unwrap()
            .try_into()
            .unwrap();

    SecretKey::from(bytes)
});
