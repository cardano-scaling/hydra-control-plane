use config::Config;
use dotenv::dotenv;
use hydra_control_plane_operator::custom_resource::HydraDoomNode;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::Level;

mod config;
mod proxy;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let state = Arc::new(State::try_new()?);
    proxy::start(state.clone()).await;
    Ok(())
}

pub struct State {
    config: Config,
    host_regex: Regex,
    nodes: RwLock<HashMap<String, HydraDoomNode>>,
}
impl State {
    pub fn try_new() -> Result<Self, Box<dyn Error>> {
        let config = Config::new();
        let host_regex = Regex::new(r"([\w\d-]+)?\.?.+")?;
        let nodes = Default::default();

        Ok(Self {
            config,
            host_regex,
            nodes,
        })
    }

    pub async fn get_node(&self, key: &str) -> Option<HydraDoomNode> {
        self.nodes.read().await.clone().get(key).cloned()
    }
}
