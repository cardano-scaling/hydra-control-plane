use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            ConfigMap, ConfigMapVolumeSource, Container, ContainerPort, EmptyDirVolumeSource,
            PodSpec, PodTemplateSpec, ResourceRequirements, SecretVolumeSource, Service,
            ServicePort, ServiceSpec, Volume, VolumeMount,
        },
        networking::v1::{
            HTTPIngressPath, HTTPIngressRuleValue, Ingress, IngressBackend, IngressRule,
            IngressServiceBackend, IngressSpec, ServiceBackendPort,
        },
    },
    apimachinery::pkg::api::resource::Quantity,
};
use kube::{api::ObjectMeta, CustomResource, ResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::Config;

use super::controller::K8sConstants;

pub static HYDRA_DOOM_NODE_FINALIZER: &str = "hydradoomnode/finalizer";

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
impl Default for Resources {
    fn default() -> Self {
        Resources {
            requests: ResourcesInner {
                cpu: "2".to_string(),
                memory: "4Gi".to_string(),
            },
            limits: ResourcesInner {
                cpu: "2".to_string(),
                memory: "4Gi".to_string(),
            },
        }
    }
}
impl From<Resources> for ResourceRequirements {
    fn from(value: Resources) -> Self {
        ResourceRequirements {
            requests: Some((&value.requests).into()),
            limits: Some((&value.limits).into()),
            ..Default::default()
        }
    }
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
        {"name": "State", "jsonPath":".status.state", "type": "string"}, 
        {"name": "Transactions", "jsonPath":".status.transactions", "type": "string"}, 
        {"name": "Local URI", "jsonPath":".status.localUrl", "type": "string"}, 
        {"name": "External URI", "jsonPath": ".status.externalUrl", "type": "string"}
    "#)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomNodeSpec {
    pub offline: Option<bool>,
    pub network_id: Option<u8>,
    pub seed_input: String,
    pub commit_inputs: Vec<String>,
    pub start_chain_from: Option<String>,
    pub asleep: Option<bool>,
    pub resources: Option<Resources>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomNodeStatus {
    pub local_url: String,
    pub external_url: String,
    pub state: String,
    pub transactions: i64,
}
impl HydraDoomNodeStatus {
    pub fn offline(crd: &HydraDoomNode, config: &Config, constants: &K8sConstants) -> Self {
        Self {
            state: "Offline".to_string(),
            transactions: 0,
            local_url: format!("ws://{}:{}", crd.internal_host(), constants.port),
            external_url: format!(
                "ws://{}:{}",
                crd.external_host(config, constants),
                config.external_port
            ),
        }
    }
}

impl HydraDoomNode {
    pub fn internal_name(&self) -> String {
        format!("hydra-doom-node-{}", self.name_any())
    }

    pub fn internal_labels(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("component".to_string(), "hydra-doom-node".to_string()),
            ("hydra-doom-node-id".to_string(), self.name_any()),
            ("run-on".to_string(), "fargate".to_string()),
        ])
    }

    pub fn internal_host(&self) -> String {
        format!(
            "{}.{}.svc.cluster.local",
            self.internal_name(),
            self.namespace().unwrap(),
        )
    }

    pub fn external_host(&self, config: &Config, _constants: &K8sConstants) -> String {
        format!("{}.{}", self.name_any(), config.external_domain)
    }

    pub fn configmap(&self, config: &Config, _constants: &K8sConstants) -> ConfigMap {
        let name = self.internal_name();

        ConfigMap {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            data: Some(BTreeMap::from([(
                "utxo.json".to_string(),
                format!(
                    r#"{{
                    "0000000000000000000000000000000000000000000000000000000000000000#0": {{
                        "address": "{}",
                        "value": {{
                            "lovelace": 1000000000
                        }}
                    }}
                }}"#,
                    config.admin_addr.clone()
                ),
            )])),
            ..Default::default()
        }
    }

    pub fn deployment(&self, config: &Config, constants: &K8sConstants) -> Deployment {
        let name = self.internal_name();
        let labels = self.internal_labels();

        // Common deployment parts:
        let main_container_common_args = vec![
            "--host".to_string(),
            "0.0.0.0".to_string(),
            "--api-host".to_string(),
            "0.0.0.0".to_string(),
            "--port".to_string(),
            "5001".to_string(),
            "--api-port".to_string(),
            constants.port.to_string(),
            "--hydra-signing-key".to_string(),
            format!("{}/hydra.sk", constants.data_dir),
            "--ledger-protocol-parameters".to_string(),
            format!("{}/protocol-parameters.json", constants.config_dir),
            "--persistence-dir".to_string(),
            constants.persistence_dir.clone(),
        ];

        let main_container_args = if self.spec.offline.unwrap_or(false) {
            let mut aux = vec![
                "offline".to_string(),
                "--initial-utxo".to_string(),
                format!("{}/utxo.json", constants.initial_utxo_config_dir),
            ];
            aux.extend(main_container_common_args);
            aux
        } else {
            let mut aux = vec![
                "--node-id".to_string(),
                self.name_any(),
                "--cardano-signing-key".to_string(),
                format!("{}/admin.sk", constants.secret_dir),
                "--hydra-scripts-tx-id".to_string(),
                config.hydra_scripts_tx_id.clone(),
                "--testnet-magic".to_string(),
                "1".to_string(), // TODO: Hardcoded preprod.
                "--node-socket".to_string(),
                constants.socket_path.clone(),
            ];
            aux.extend(main_container_common_args);
            if let Some(start_chain_from) = &self.spec.start_chain_from {
                aux.push("--start-chain-from".to_string());
                aux.push(start_chain_from.clone());
            }
            aux
        };

        let mut containers = vec![
            Container {
                name: "main".to_string(),
                image: Some(config.image.clone()),
                args: Some(main_container_args),
                ports: Some(vec![ContainerPort {
                    name: Some("api".to_string()),
                    container_port: constants.port,
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                volume_mounts: Some(vec![
                    VolumeMount {
                        name: "initialutxo".to_string(),
                        mount_path: constants.initial_utxo_config_dir.clone(),
                        ..Default::default()
                    },
                    VolumeMount {
                        name: "config".to_string(),
                        mount_path: constants.config_dir.clone(),
                        ..Default::default()
                    },
                    VolumeMount {
                        name: "data".to_string(),
                        mount_path: constants.data_dir.clone(),
                        ..Default::default()
                    },
                    VolumeMount {
                        name: "secret".to_string(),
                        mount_path: constants.secret_dir.clone(),
                        ..Default::default()
                    },
                    VolumeMount {
                        name: "ipc".to_string(),
                        mount_path: constants.socket_dir.clone(),
                        ..Default::default()
                    },
                ]),
                resources: Some(
                    self.spec
                        .resources
                        .clone()
                        .unwrap_or(Default::default())
                        .into(),
                ),
                ..Default::default()
            },
            Container {
                name: "sidecar".to_string(),
                image: Some(config.sidecar_image.clone()),
                args: Some(vec![
                    "metrics-exporter".to_string(),
                    "--host".to_string(),
                    "localhost".to_string(),
                    "--port".to_string(),
                    constants.port.to_string(),
                ]),
                ports: Some(vec![ContainerPort {
                    name: Some("metrics".to_string()),
                    container_port: constants.metrics_port,
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        ];

        // Offline is optional. If undefined, the node is presumed to be online.
        if !self.spec.offline.unwrap_or(false) {
            let mut open_head_args = vec![
                "open-head".to_string(),
                "--network-id".to_string(),
                self.spec.network_id.unwrap_or(0).to_string(),
                "--seed-input".to_string(),
                self.spec.seed_input.clone(),
                "--participant".to_string(),
                config.admin_addr.clone(),
                "--party-verification-file".to_string(),
                format!("{}/hydra.vk", constants.data_dir),
                "--cardano-key-file".to_string(),
                format!("{}/admin.sk", constants.secret_dir),
                "--blockfrost-key".to_string(),
                config.blockfrost_key.clone(),
            ];
            if !self.spec.commit_inputs.is_empty() {
                open_head_args.push("--commit-inputs".to_string());
                open_head_args.extend(self.spec.commit_inputs.clone());
            }

            containers.push(Container {
                name: "open-head".to_string(),
                image: Some(config.open_head_image.clone()),
                args: Some(open_head_args),
                volume_mounts: Some(vec![
                    VolumeMount {
                        name: "config".to_string(),
                        mount_path: constants.config_dir.clone(),
                        ..Default::default()
                    },
                    VolumeMount {
                        name: "secret".to_string(),
                        mount_path: constants.secret_dir.clone(),
                        ..Default::default()
                    },
                    VolumeMount {
                        name: "data".to_string(),
                        mount_path: constants.data_dir.clone(),
                        ..Default::default()
                    },
                ]),
                resources: None,
                ..Default::default()
            });

            containers.push(Container {
                name: "dmtrctl".to_string(),
                image: Some(constants.dmtrctl_image.to_string()),
                args: Some(vec![
                    "--project-id".to_string(),
                    config.dmtr_project_id.clone(),
                    "--api-key".to_string(),
                    config.dmtr_api_key.clone(),
                    "ports".to_string(),
                    "tunnel".to_string(),
                    config.dmtr_port_name.clone(),
                    "--socket".to_string(),
                    constants.socket_path.clone(),
                ]),
                volume_mounts: Some(vec![VolumeMount {
                    name: "ipc".to_string(),
                    mount_path: constants.socket_dir.clone(),
                    ..Default::default()
                }]),
                ..Default::default()
            })
        }

        Deployment {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(if self.spec.asleep.unwrap_or(false) {
                    0
                } else {
                    1
                }),
                selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                    match_labels: Some(labels.clone()),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels.clone()),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        init_containers: Some(vec![Container {
                            name: "init".to_string(),
                            image: Some(config.image.clone()),
                            args: Some(vec![
                                "gen-hydra-key".to_string(),
                                "--output-file".to_string(),
                                format!("{}/hydra", constants.data_dir),
                            ]),
                            volume_mounts: Some(vec![VolumeMount {
                                name: "data".to_string(),
                                mount_path: constants.data_dir.clone(),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        }]),
                        containers,
                        volumes: Some(vec![
                            Volume {
                                name: "data".to_string(),
                                empty_dir: Some(EmptyDirVolumeSource::default()),
                                ..Default::default()
                            },
                            Volume {
                                name: "secret".to_string(),
                                secret: Some(SecretVolumeSource {
                                    secret_name: Some(config.secret.clone()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            Volume {
                                name: "config".to_string(),
                                config_map: Some(ConfigMapVolumeSource {
                                    name: config.configmap.clone(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            Volume {
                                name: "initialutxo".to_string(),
                                config_map: Some(ConfigMapVolumeSource {
                                    name: name.clone(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            Volume {
                                name: "ipc".to_string(),
                                empty_dir: Some(EmptyDirVolumeSource::default()),
                                ..Default::default()
                            },
                        ]),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn service(&self, _config: &Config, constants: &K8sConstants) -> Service {
        let name = self.internal_name();
        let labels = self.internal_labels();
        Service {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some(labels),
                ports: Some(vec![
                    ServicePort {
                        name: Some("websocket".to_string()),
                        port: constants.port,
                        target_port: Some(
                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                                constants.port,
                            ),
                        ),
                        protocol: Some("TCP".to_string()),
                        ..Default::default()
                    },
                    ServicePort {
                        name: Some("metrics".to_string()),
                        port: constants.metrics_port,
                        target_port: Some(
                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                                constants.metrics_port,
                            ),
                        ),
                        protocol: Some("TCP".to_string()),
                        ..Default::default()
                    },
                ]),
                type_: Some("ClusterIP".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn ingress(&self, config: &Config, constants: &K8sConstants) -> Ingress {
        let name = self.internal_name();
        Ingress {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                annotations: Some(constants.ingress_annotations.clone()),
                ..Default::default()
            },
            spec: Some(IngressSpec {
                ingress_class_name: Some(constants.ingress_class_name.clone()),
                rules: Some(vec![IngressRule {
                    host: Some(self.external_host(config, constants)),
                    http: Some(HTTPIngressRuleValue {
                        paths: vec![HTTPIngressPath {
                            path: Some("/".to_string()),
                            path_type: "Prefix".to_string(),
                            backend: IngressBackend {
                                service: Some(IngressServiceBackend {
                                    name: name.clone(),
                                    port: Some(ServiceBackendPort {
                                        number: Some(constants.port),
                                        ..Default::default()
                                    }),
                                }),
                                ..Default::default()
                            },
                        }],
                    }),
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
