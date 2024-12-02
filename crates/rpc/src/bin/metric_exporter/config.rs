use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub hydra_pod_name: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            hydra_pod_name: env::var("HYDRA_POD_NAME").expect("HYDRA_POD_NAME must be set"),
        }
    }
}
