use lazy_static::lazy_static;
use std::env;

lazy_static! {
    static ref CONTROLLER_CONFIG: Config = Config::from_env();
}

pub fn get_config() -> &'static Config {
    &CONTROLLER_CONFIG
}

#[derive(Debug, Clone)]
pub struct Config {
    pub image: String,
    pub open_head_image: String,
    pub sidecar_image: String,
    pub configmap: String,
    pub secret: String,
    pub blockfrost_key: String,
    pub external_domain: String,
    pub external_port: String,
    pub external_protocol: String,
    pub admin_addr: String,
    pub hydra_scripts_tx_id: String,
    pub dmtr_project_id: String,
    pub dmtr_api_key: String,
    pub dmtr_port_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            image: env::var("IMAGE").unwrap_or("ghcr.io/cardano-scaling/hydra-node".into()),
            open_head_image: env::var("OPEN_HEAD_IMAGE").expect("Missing OPEN_HEAD_IMAGE env var"),
            sidecar_image: env::var("SIDECAR_IMAGE").expect("Missing SIDECAR_IMAGE env var"),
            configmap: env::var("CONFIGMAP").expect("Missing CONFIGMAP env var"),
            secret: env::var("SECRET").expect("Missing SECRET env var"),
            blockfrost_key: env::var("BLOCKFROST_KEY").expect("Missing BLOCKFROST_KEY env var"),
            external_domain: env::var("EXTERNAL_DOMAIN").expect("Missing EXTERNAL_DOMAIN env var."),
            external_port: env::var("EXTERNAL_PORT").expect("Missing EXTERNAL_PORT env var."),
            external_protocol: env::var("EXTERNAL_PROTOCOL")
                .expect("Missing EXTERNAL_PROTOCOL env var."),
            admin_addr: env::var("ADMIN_ADDR").expect("Missing ADMIN_ADDR env var."),
            hydra_scripts_tx_id: env::var("HYDRA_SCRIPTS_TX_ID")
                .expect("Missing HYDRA_SCRIPTS_TX_ID env var."),
            dmtr_project_id: env::var("DMTR_PROJECT_ID").expect("Missing DMTR_PROJECT_ID env var."),
            dmtr_api_key: env::var("DMTR_API_KEY").expect("Missing DMTR_API_KEY env var."),
            dmtr_port_name: env::var("DMTR_PORT_NAME").expect("Missing DMTR_PORT_NAME env var."),
        }
    }
}
