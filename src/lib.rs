use std::path::PathBuf;

use ::serde::Deserialize;

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
