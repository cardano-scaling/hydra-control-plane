use lazy_static::lazy_static;
use std::{env, time::Duration};

lazy_static! {
    static ref CONTROLLER_CONFIG: Config = Config::from_env();
}

pub fn get_config() -> &'static Config {
    &CONTROLLER_CONFIG
}

#[derive(Debug, Clone)]
pub struct Config {
    pub image: String,
    pub init_image: String,
    pub sidecar_image: String,
    pub referee_image: String,
    pub ai_image: String,
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
    pub bucket: String,
    pub bucket_region: String,
    pub init_aws_access_key_id: String,
    pub init_aws_secret_access_key: String,
    pub network_id: String,
    pub available_snapshot_prefix: String,

    // Autoscaler
    pub autoscaler_delay: Duration,
    pub autoscaler_low_watermark: usize,
    pub autoscaler_high_watermark: usize,
    pub autoscaler_region_prefix: String,
    pub autoscaler_max_batch: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            image: env::var("IMAGE").unwrap_or("ghcr.io/cardano-scaling/hydra-node".into()),
            sidecar_image: env::var("SIDECAR_IMAGE").expect("Missing SIDECAR_IMAGE env var"),
            referee_image: env::var("REFEREE_IMAGE").expect("Missing REFEREE_IMAGE env var"),
            ai_image: env::var("AI_IMAGE").expect("Missing AI_IMAGE env var"),
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
            init_image: env::var("INIT_IMAGE").expect("Missing INIT_IMAGE env var."),
            bucket: env::var("BUCKET").expect("Missing BUCKET env var."),
            bucket_region: env::var("BUCKET_REGION").expect("Missing BUCKET_REGION env var."),
            init_aws_access_key_id: env::var("INIT_AWS_ACCESS_KEY_ID")
                .expect("Missing INIT_AWS_ACCESS_KEY_ID env var."),
            init_aws_secret_access_key: env::var("INIT_AWS_SECRET_ACCESS_KEY")
                .expect("Missing INIT_AWS_SECRET_ACCESS_KEY env var."),
            available_snapshot_prefix: env::var("AVAILABLE_SNAPSHOT_PREFIX")
                .unwrap_or("snapshots".to_string()),

            autoscaler_delay: env::var("AUTOSCALER_DELAY")
                .map(|duration| {
                    Duration::from_secs(duration.parse().expect("Failed to parse AUTOSCALER_DELAY"))
                })
                .expect("Missing AUTOSCALER_DELAY env var."),
            autoscaler_high_watermark: env::var("AUTOSCALER_HIGH_WATERMARK")
                .map(|x| {
                    x.parse()
                        .expect("Failed to parse AUTOSCALER_HIGH_WATERMARK")
                })
                .expect("Missing AUTOSCALER_HIGH_WATERMARK env var."),
            autoscaler_low_watermark: env::var("AUTOSCALER_LOW_WATERMARK")
                .map(|x| x.parse().expect("Failed to parse AUTOSCALER_LOW_WATERMARK"))
                .expect("Missing AUTOSCALER_LOW_WATERMARK env var."),
            autoscaler_region_prefix: env::var("AUTOSCALER_REGION_PREFIX")
                .expect("Missing AUTOSCALER_REGION_PREFIX env var."),
            autoscaler_max_batch: env::var("AUTOSCALER_MAX_BATCH")
                .map(|x| x.parse().expect("Failed to parse AUTOSCALER_MAX_BATCH"))
                .expect("Missing AUTOSCALER_MAX_BATCH env var."),
            network_id: env::var("NETWORK_ID").expect("Missing NETWORK_ID env var."),
        }
    }
}
