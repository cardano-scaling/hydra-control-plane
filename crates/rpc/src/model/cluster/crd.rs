use std::collections::BTreeMap;

use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ResourcesInner {
    pub cpu: String,
    pub memory: String,
}
impl From<&ResourcesInner> for BTreeMap<String, Quantity> {
    fn from(value: &ResourcesInner) -> Self {
        BTreeMap::from([
            ("cpu".to_string(), Quantity(value.cpu.clone())),
            ("memory".to_string(), Quantity(value.memory.clone())),
        ])
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct Resources {
    pub requests: ResourcesInner,
    pub limits: ResourcesInner,
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "HydraDoomNode",
    group = "hydra.doom",
    version = "v1alpha1",
    shortname = "hydradoomnode",
    category = "hydradoom",
    plural = "hydradoomnodes",
    namespaced
)]
#[kube(status = "HydraDoomNodeStatus")]
#[kube(printcolumn = r#"
        {"name": "Node State", "jsonPath":".status.nodeState", "type": "string"},
        {"name": "Game State", "jsonPath":".status.gameState", "type": "string"},
        {"name": "Transactions", "jsonPath":".status.transactions", "type": "string"},
        {"name": "Local URI", "jsonPath":".status.localUrl", "type": "string"},
        {"name": "External URI", "jsonPath": ".status.externalUrl", "type": "string"}
    "#)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomNodeSpec {
    pub offline: Option<bool>,
    pub network_id: Option<u8>,
    pub snapshot: Option<String>,
    pub start_chain_from: Option<String>,
    pub asleep: Option<bool>,
    pub resources: Option<Resources>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomNodeStatus {
    pub local_url: String,
    pub external_url: String,
    pub node_state: String,
    pub game_state: String,
    pub transactions: i64,
}
