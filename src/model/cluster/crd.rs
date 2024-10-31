use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
        {"name": "Local URI", "jsonPath":".status.localUrl", "type": "string"}, 
        {"name": "External URI", "jsonPath": ".status.externalUrl", "type": "string"}
    "#)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomNodeSpec {
    pub image: Option<String>,
    pub open_head_image: Option<String>,
    pub configmap: Option<String>,
    pub network_id: u8,
    pub seed_input: String,
    pub participant: String,
    pub party: String,
    pub commit_inputs: Vec<String>,
    pub blockfrost_key: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomNodeStatus {
    pub local_url: String,
    pub external_url: String,
}
