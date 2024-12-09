use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            ConfigMap, ConfigMapVolumeSource, Container, ContainerPort, EmptyDirVolumeSource,
            EnvVar, PodSpec, PodTemplateSpec, Probe, ResourceRequirements, SecretVolumeSource,
            Service, ServicePort, ServiceSpec, Volume, VolumeMount,
        },
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::OwnerReference},
};
use kube::{api::ObjectMeta, CustomResource, Resource, ResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::Config;

use super::controller::K8sConstants;

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
                cpu: "4".to_string(),
                memory: "6Gi".to_string(),
            },
            limits: ResourcesInner {
                cpu: "4".to_string(),
                memory: "6Gi".to_string(),
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
        {"name": "Node State", "jsonPath":".status.nodeState", "type": "string"},
        {"name": "Game State", "jsonPath":".status.gameState", "type": "string"},
        {"name": "Snapshot", "jsonPath":".spec.snapshot", "type": "string"},
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

impl Default for HydraDoomNodeSpec {
    fn default() -> Self {
        Self {
            offline: Some(true),
            network_id: None,
            snapshot: None,
            start_chain_from: None,
            asleep: None,
            resources: None,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomNodeStatus {
    pub local_url: String,
    pub external_url: String,
    pub node_state: String,
    pub game_state: String,
}
impl HydraDoomNodeStatus {
    pub fn offline(crd: &HydraDoomNode, config: &Config, constants: &K8sConstants) -> Self {
        Self {
            node_state: "Offline".to_string(),
            game_state: "Done".to_string(),
            local_url: format!("ws://{}:{}", crd.internal_host(), constants.port),
            external_url: format!(
                "{}://{}:{}",
                config.external_protocol,
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

    pub fn owner_references(&self) -> Vec<OwnerReference> {
        vec![OwnerReference {
            api_version: HydraDoomNode::api_version(&()).to_string(),
            kind: HydraDoomNode::kind(&()).to_string(),
            name: self.name_any(),
            uid: self.uid().unwrap(),
            ..Default::default()
        }]
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
                owner_references: Some(self.owner_references()),
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
            format!("{}/keys/hydra.sk", constants.data_dir),
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
                "--ledger-genesis".to_string(),
                format!("{}/shelley-genesis.json", constants.config_dir),
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
                "--node-socket".to_string(),
                constants.socket_path.clone(),
            ];
            aux.extend(main_container_common_args);
            if let Some(start_chain_from) = &self.spec.start_chain_from {
                aux.push("--start-chain-from".to_string());
                aux.push(start_chain_from.clone());
            }

            if config.network_id == "0" {
                aux.push("--testnet-magic".to_string());
                aux.push("1".to_string());
            } else {
                // Assume mainnet in any other case.
                aux.push("--mainnet".to_string())
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
                readiness_probe: Some(Probe {
                    tcp_socket: Some(k8s_openapi::api::core::v1::TCPSocketAction {
                        port: k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                            constants.port,
                        ),
                        ..Default::default()
                    }),
                    initial_delay_seconds: Some(15),
                    period_seconds: Some(10),
                    ..Default::default()
                }),
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
                resources: Some(self.spec.resources.clone().unwrap_or_default().into()),
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
                    "--admin-key-file".to_string(),
                    format!("{}/admin.sk", constants.secret_dir),
                ]),
                volume_mounts: Some(vec![VolumeMount {
                    name: "secret".to_string(),
                    mount_path: constants.secret_dir.clone(),
                    ..Default::default()
                }]),
                env: Some(vec![EnvVar {
                    name: "NETWORK_ID".to_string(),
                    value: Some(config.network_id.clone()),
                    value_from: None,
                }]),
                ports: Some(vec![ContainerPort {
                    name: Some("metrics".to_string()),
                    container_port: constants.metrics_port,
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            Container {
                name: "referee".to_string(),
                image: Some(config.referee_image.clone()),
                env: Some(vec![
                    EnvVar {
                        name: "ADMIN_KEY_FILE".to_string(),
                        value: Some(format!("{}/admin.sk", constants.secret_dir)),
                        value_from: None,
                    },
                    EnvVar {
                        name: "NETWORK_ID".to_string(),
                        value: Some(config.network_id.clone()),
                        value_from: None,
                    },
                ]),
                volume_mounts: Some(vec![VolumeMount {
                    name: "secret".to_string(),
                    mount_path: constants.secret_dir.clone(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            Container {
                name: "ai-1".to_string(),
                image: Some(config.ai_image.clone()),
                env: Some(vec![
                    EnvVar {
                        name: "NETWORK_ID".to_string(),
                        value: Some(config.network_id.clone()),
                        value_from: None,
                    },
                    EnvVar {
                        name: "ADMIN_KEY_FILE".to_string(),
                        value: Some(format!("{}/admin.sk", constants.secret_dir)),
                        value_from: None,
                    },
                    EnvVar {
                        name: "BOT_INDEX".to_string(),
                        value: Some("1".to_string()),
                        value_from: None,
                    },
                ]),
                volume_mounts: Some(vec![VolumeMount {
                    name: "secret".to_string(),
                    mount_path: constants.secret_dir.clone(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            Container {
                name: "ai-2".to_string(),
                image: Some(config.ai_image.clone()),
                env: Some(vec![
                    EnvVar {
                        name: "NETWORK_ID".to_string(),
                        value: Some(config.network_id.clone()),
                        value_from: None,
                    },
                    EnvVar {
                        name: "ADMIN_KEY_FILE".to_string(),
                        value: Some(format!("{}/admin.sk", constants.secret_dir)),
                        value_from: None,
                    },
                    EnvVar {
                        name: "BOT_INDEX".to_string(),
                        value: Some("2".to_string()),
                        value_from: None,
                    },
                ]),
                volume_mounts: Some(vec![VolumeMount {
                    name: "secret".to_string(),
                    mount_path: constants.secret_dir.clone(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            Container {
                name: "ai-3".to_string(),
                image: Some(config.ai_image.clone()),
                env: Some(vec![
                    EnvVar {
                        name: "NETWORK_ID".to_string(),
                        value: Some(config.network_id.clone()),
                        value_from: None,
                    },
                    EnvVar {
                        name: "ADMIN_KEY_FILE".to_string(),
                        value: Some(format!("{}/admin.sk", constants.secret_dir)),
                        value_from: None,
                    },
                    EnvVar {
                        name: "BOT_INDEX".to_string(),
                        value: Some("3".to_string()),
                        value_from: None,
                    },
                ]),
                volume_mounts: Some(vec![VolumeMount {
                    name: "secret".to_string(),
                    mount_path: constants.secret_dir.clone(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        ];

        // Offline is optional. If undefined, the node is presumed to be online.
        if !self.spec.offline.unwrap_or(false) {
            let node_service = match config.network_id.as_str() {
                "0" => "node-preprod".to_string(),
                _ => "node-mainnet".to_string(),
            };

            containers.push(Container {
                name: "socat".to_string(),
                image: Some(constants.socat_image.to_string()),
                args: Some(vec![
                    format!("UNIX-LISTEN:{0},fork", constants.socket_path),
                    format!(
                        "TCP:{0}.hydra-doom-system.svc.cluster.local:3307,ignoreeof",
                        node_service
                    ),
                ]),
                volume_mounts: Some(vec![VolumeMount {
                    name: "ipc".to_string(),
                    mount_path: constants.socket_dir.clone(),
                    ..Default::default()
                }]),
                ..Default::default()
            })
        }

        let mut init_container_env_vars = vec![
            EnvVar {
                name: "BUCKET".to_string(),
                value: Some(config.bucket.clone()),
                ..Default::default()
            },
            EnvVar {
                name: "DATA_DIR".to_string(),
                value: Some(constants.data_dir.clone()),
                ..Default::default()
            },
            EnvVar {
                name: "AWS_REGION".to_string(),
                value: Some(config.bucket_region.clone()),
                ..Default::default()
            },
            EnvVar {
                name: "AWS_ACCESS_KEY_ID".to_string(),
                value: Some(config.init_aws_access_key_id.clone()),
                ..Default::default()
            },
            EnvVar {
                name: "AWS_SECRET_ACCESS_KEY".to_string(),
                value: Some(config.init_aws_secret_access_key.clone()),
                ..Default::default()
            },
        ];
        if let Some(key) = &self.spec.snapshot {
            init_container_env_vars.push(EnvVar {
                name: "KEY".to_string(),
                value: Some(key.clone()),
                ..Default::default()
            });
        }

        Deployment {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                owner_references: Some(self.owner_references()),
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
                        service_account_name: Some(constants.service_account_name.clone()),
                        init_containers: Some(vec![Container {
                            name: "init".to_string(),
                            image: Some(config.init_image.clone()),
                            env: Some(init_container_env_vars),
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
                owner_references: Some(self.owner_references()),
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
}
