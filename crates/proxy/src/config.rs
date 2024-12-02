use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub proxy_addr: String,
    pub hydra_node_port: u16,
    pub hydra_node_dns: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            proxy_addr: env::var("PROXY_ADDR").expect("PROXY_ADDR must be set"),
            hydra_node_port: env::var("HYDRA_NODE_PORT")
                .expect("HYDRA_NODE_PORT must be set")
                .parse()
                .expect("HYDRA_NODE_PORT must a number"),
            hydra_node_dns: env::var("HYDRA_NODE_DNS").expect("HYDRA_NODE_DNS must be set"),
        }
    }
}
