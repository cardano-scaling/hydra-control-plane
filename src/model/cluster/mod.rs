use std::{path::PathBuf, sync::Arc};

use serde::Deserialize;
use tokio::sync::RwLock;

pub mod metrics;
mod node;

pub use node::*;

#[derive(Clone)]
pub struct ClusterState {}

impl ClusterState {
    pub async fn in_cluster() -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub async fn remote(k8s_api_url: String) -> anyhow::Result<Self> {
        todo!()
    }

    pub async fn get_warm_node(&self) -> anyhow::Result<Node> {
        todo!()
    }

    pub async fn get_all_nodes(&self) -> Vec<Node> {
        todo!()
    }

    pub async fn get_node_by_id(&self, id: &str) -> anyhow::Result<Option<Node>> {
        todo!()
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ttl_minutes: u64,
}

#[derive(Debug, Deserialize)]
struct HostConfig {
    #[serde(default = "localhost")]
    local_url: String,
    remote_url: Option<String>,

    max_players: usize,
    admin_key_file: PathBuf,
    persisted: bool,
    reserved: bool,
}

fn default_start_port() -> u32 {
    4001
}

fn default_region() -> String {
    "us-east-2".to_string()
}

fn localhost() -> String {
    "ws://127.0.0.1".to_string()
}
