use anyhow::bail;
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{ConfigMap, Service},
    networking::v1::Ingress,
};
use kube::{
    api::{DeleteParams, ListParams, Patch, PatchParams},
    runtime::controller::Action,
    Api, Client, ResourceExt,
};
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use thiserror::Error;
use tracing::{error, info, warn};

use crate::{config::Config, custom_resource::HydraDoomNodeStatus};

use super::custom_resource::{HydraDoomNode, HYDRA_DOOM_NODE_FINALIZER};

pub enum HydraDoomNodeState {
    Offline,
    Online,
    HeadIsInitializing,
    HeadIsOpen,
    Sleeping,
}
impl From<f64> for HydraDoomNodeState {
    fn from(value: f64) -> Self {
        match value {
            1.0 => Self::Online,
            2.0 => Self::HeadIsInitializing,
            3.0 => Self::HeadIsOpen,
            _ => Self::Offline,
        }
    }
}
impl From<HydraDoomNodeState> for String {
    fn from(val: HydraDoomNodeState) -> Self {
        match val {
            HydraDoomNodeState::Offline => "Offline".to_string(),
            HydraDoomNodeState::Online => "Online".to_string(),
            HydraDoomNodeState::HeadIsInitializing => "HeadIsInitializing".to_string(),
            HydraDoomNodeState::HeadIsOpen => "HeadIsOpen".to_string(),
            HydraDoomNodeState::Sleeping => "Sleeping".to_string(),
        }
    }
}

pub struct K8sConstants {
    pub config_dir: String,
    pub secret_dir: String,
    pub socket_dir: String,
    pub socket_path: String,
    pub initial_utxo_config_dir: String,
    pub data_dir: String,
    pub persistence_dir: String,
    pub node_port: i32,
    pub port: i32,
    pub ingress_class_name: String,
    pub ingress_annotations: BTreeMap<String, String>,
    pub metrics_port: i32,
    pub metrics_endpoint: String,
    pub state_metric: String,
    pub transactions_metric: String,
    pub dmtrctl_image: String,
    pub storage_class_name: String,
}
impl Default for K8sConstants {
    fn default() -> Self {
        Self {
            storage_class_name: "efs-sc".to_string(),
            config_dir: "/etc/config".to_string(),
            secret_dir: "/var/secret".to_string(),
            socket_dir: "/ipc".to_string(),
            dmtrctl_image: "ghcr.io/demeter-run/dmtrctl:sha-3ffefaa".to_string(),
            socket_path: "/ipc/socket".to_string(),
            initial_utxo_config_dir: "/etc/initial_utxo_config".to_string(),
            data_dir: "/var/data".to_string(),
            persistence_dir: "/var/data/persistence".to_string(),
            node_port: 5001,
            port: 4001,
            metrics_port: 8000,
            metrics_endpoint: "/metrics".to_string(),
            state_metric: "hydra_doom_node_state".to_string(),
            transactions_metric: "hydra_doom_node_transactions".to_string(),
            ingress_class_name: "nginx".to_string(),
            ingress_annotations: [
                (
                    "nginx.ingress.kubernetes.io/proxy-read-timeout".to_string(),
                    "3600".to_string(),
                ),
                (
                    "nginx.ingress.kubernetes.io/proxy-send-timeout".to_string(),
                    "3600".to_string(),
                ),
                (
                    "nginx.ingress.kubernetes.io/server-snippets".to_string(),
                    "location / {\n\
                      proxy_set_header Upgrade $http_upgrade;\n\
                      proxy_http_version 1.1;\n\
                      proxy_set_header X-Forwarded-Host $http_host;\n\
                      proxy_set_header X-Forwarded-Proto $scheme;\n\
                      proxy_set_header X-Forwarded-For $remote_addr;\n\
                      proxy_set_header Host $host;\n\
                      proxy_set_header Connection \"upgrade\";\n\
                      proxy_cache_bypass $http_upgrade;\n\
                    }\n"
                    .to_string(),
                ),
            ]
            .into(),
        }
    }
}

pub struct K8sContext {
    pub client: Client,
    pub config: Config,
    pub constants: K8sConstants,
}

impl K8sContext {
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            client,
            config,
            constants: Default::default(),
        }
    }

    pub async fn patch(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        info!("Running patch");
        match tokio::join!(
            self.patch_deployment(crd),
            self.patch_service(crd),
            self.patch_ingress(crd),
            self.patch_configmap(crd),
            self.patch_crd(crd)
        ) {
            (Ok(_), Ok(_), Ok(_), Ok(_), Ok(_)) => (),
            _ => bail!("Failed to apply patch for components."),
        };

        Ok(())
    }

    pub async fn delete(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        match tokio::join!(
            self.remove_deployment(crd),
            self.remove_service(crd),
            self.remove_ingress(crd),
            self.remove_configmap(crd),
        ) {
            (Ok(_), Ok(_), Ok(_), Ok(_)) => Ok(()),
            _ => bail!("Failed to remove resources"),
        }
    }

    async fn patch_crd(&self, crd: &HydraDoomNode) -> anyhow::Result<HydraDoomNode> {
        let api: Api<HydraDoomNode> =
            Api::namespaced(self.client.clone(), &crd.namespace().unwrap());

        api.patch(
            &crd.name_any(),
            &PatchParams::default(),
            &Patch::Merge(json!({
                "metadata": {
                    "finalizers": [HYDRA_DOOM_NODE_FINALIZER]
                }
            })),
        )
        .await
        .map_err(|err| {
            error!(err = err.to_string(), "Failed to patch CRD.");
            anyhow::Error::from(err)
        })
    }

    async fn patch_configmap(&self, crd: &HydraDoomNode) -> anyhow::Result<ConfigMap> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), &crd.namespace().unwrap());

        // Create or patch the configmap
        api.patch(
            &crd.internal_name(),
            &PatchParams::apply("hydra-doom-pod-controller"),
            &Patch::Apply(&crd.configmap(&self.config, &self.constants)),
        )
        .await
        .map_err(|err| {
            error!(err = err.to_string(), "Failed to create configmap.");
            err.into()
        })
    }

    async fn remove_configmap(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        match api
            .delete(&crd.internal_name(), &DeleteParams::default())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn patch_deployment(&self, crd: &HydraDoomNode) -> anyhow::Result<Deployment> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), &crd.namespace().unwrap());

        // Create or patch the deployment
        api.patch(
            &crd.internal_name(),
            &PatchParams::apply("hydra-doom-pod-controller"),
            &Patch::Apply(&crd.deployment(&self.config, &self.constants)),
        )
        .await
        .map_err(|err| {
            error!(err = err.to_string(), "Failed to create deployment.");
            err.into()
        })
    }

    async fn remove_deployment(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        let dp = DeleteParams::default();

        match api.delete(&crd.internal_name(), &dp).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn patch_service(&self, crd: &HydraDoomNode) -> anyhow::Result<Service> {
        // Apply the service to the cluster
        let services: Api<Service> =
            Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        services
            .patch(
                &crd.internal_name(),
                &PatchParams::apply("hydra-doom-pod-controller"),
                &Patch::Apply(&crd.service(&self.config, &self.constants)),
            )
            .await
            .map_err(|err| {
                error!(err = err.to_string(), "Failed to create service.");
                err.into()
            })
    }

    async fn remove_service(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        let services: Api<Service> =
            Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        let dp = DeleteParams::default();
        match services.delete(&crd.internal_name(), &dp).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn patch_ingress(&self, crd: &HydraDoomNode) -> anyhow::Result<Ingress> {
        // Apply the service to the cluster
        let api: Api<Ingress> = Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        api.patch(
            &crd.internal_name(),
            &PatchParams::apply("hydra-doom-pod-controller"),
            &Patch::Apply(&crd.ingress(&self.config, &self.constants)),
        )
        .await
        .map_err(|err| {
            error!(err = err.to_string(), "Failed to create ingress.");
            err.into()
        })
    }

    async fn remove_ingress(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        let api: Api<Ingress> = Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        let dp = DeleteParams::default();
        match api.delete(&crd.internal_name(), &dp).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_status_from_crd(&self, crd: &HydraDoomNode) -> HydraDoomNodeStatus {
        let url = format!(
            "http://{}:{}{}",
            crd.internal_host(),
            self.constants.metrics_port,
            self.constants.metrics_endpoint
        );

        if crd.spec.asleep.unwrap_or(false) {
            return HydraDoomNodeStatus {
                state: HydraDoomNodeState::Sleeping.into(),
                transactions: 0,
                local_url: format!("ws://{}:{}", crd.internal_host(), self.constants.port),
                external_url: format!(
                    "ws://{}:{}",
                    crd.external_host(&self.config, &self.constants),
                    self.config.external_port
                ),
            };
        }

        let default = HydraDoomNodeStatus::offline(crd, &self.config, &self.constants);

        match reqwest::get(&url).await {
            Ok(response) => match response.text().await {
                Ok(body) => {
                    let lines: Vec<_> = body.lines().map(|s| Ok(s.to_owned())).collect();
                    match prometheus_parse::Scrape::parse(lines.into_iter()) {
                        Ok(metrics) => {
                            let state = metrics
                                .clone()
                                .samples
                                .into_iter()
                                .find(|sample| sample.metric == self.constants.state_metric)
                                .map(|sample| match sample.value {
                                    prometheus_parse::Value::Gauge(value) => {
                                        HydraDoomNodeState::from(value)
                                    }
                                    _ => HydraDoomNodeState::Offline,
                                });

                            let transactions = metrics
                                .clone()
                                .samples
                                .into_iter()
                                .find(|sample| sample.metric == self.constants.transactions_metric)
                                .map(|sample| match sample.value {
                                    prometheus_parse::Value::Counter(count) => count.round() as i64,
                                    _ => 0,
                                });

                            match (state, transactions) {
                                (Some(state), Some(transactions)) => HydraDoomNodeStatus {
                                    transactions,
                                    state: state.into(),
                                    local_url: format!(
                                        "ws://{}:{}",
                                        crd.internal_host(),
                                        self.constants.port
                                    ),
                                    external_url: format!(
                                        "ws://{}:{}",
                                        crd.external_host(&self.config, &self.constants),
                                        self.config.external_port
                                    ),
                                },
                                _ => default,
                            }
                        }
                        Err(err) => {
                            warn!(
                                err = err.to_string(),
                                "Failed to parse metrics for {}",
                                crd.name_any()
                            );
                            default
                        }
                    }
                }
                Err(err) => {
                    warn!(
                        err = err.to_string(),
                        "Failed to parse request response to metrics endpoint for {}",
                        crd.name_any()
                    );
                    default
                }
            },
            Err(err) => {
                warn!(
                    err = err.to_string(),
                    "Failed to request metrics for {}",
                    crd.name_any()
                );
                default
            }
        }
    }

    async fn patch_statuses(&self) -> anyhow::Result<()> {
        let api: Api<HydraDoomNode> = Api::default_namespaced(self.client.clone());
        let crds = api.list(&ListParams::default()).await?;

        let mut awaitables = vec![];
        for crd in &crds {
            awaitables.push(async {
                let name = crd.name_any();
                let api: Api<HydraDoomNode> =
                    Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
                if let Err(err) = api
                    .patch_status(
                        &name,
                        &PatchParams::default(),
                        &Patch::Merge(json!({ "status": self.get_status_from_crd(crd).await })),
                    )
                    .await
                {
                    warn!(
                        err = err.to_string(),
                        "Failed to update status for CRD {}.", name
                    );
                };
            })
        }

        futures::future::join_all(awaitables).await;

        Ok(())
    }
}

pub async fn patch_statuses(context: Arc<K8sContext>) -> Result<()> {
    info!("Running status patcher loop.");

    loop {
        context.patch_statuses().await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// Auxiliary error value because K8s controller api doesnt go along with anyhow.
#[derive(Debug, Error)]
pub enum Error {
    #[error("ReconcileError")]
    ReconcileError,
}
impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        error!("Reconcile error: {}", value.to_string());
        Self::ReconcileError
    }
}
type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn reconcile(crd: Arc<HydraDoomNode>, ctx: Arc<K8sContext>) -> Result<Action, Error> {
    tracing::info!("Reconciling {}", crd.name_any());
    // Check if deletion timestamp is set
    if crd.metadata.deletion_timestamp.is_some() {
        let hydra_doom_pod_api: Api<HydraDoomNode> =
            Api::namespaced(ctx.client.clone(), &crd.namespace().unwrap());
        // Finalizer logic for cleanup
        if crd
            .finalizers()
            .contains(&HYDRA_DOOM_NODE_FINALIZER.to_string())
        {
            // Delete associated resources
            ctx.delete(&crd).await?;
            // Remove finalizer
            let patch = json!({
                "metadata": {
                    "finalizers": crd.finalizers().iter().filter(|f| *f != HYDRA_DOOM_NODE_FINALIZER).collect::<Vec<_>>()
                }
            });
            let _ = hydra_doom_pod_api
                .patch(
                    &crd.name_any(),
                    &PatchParams::default(),
                    &Patch::Merge(&patch),
                )
                .await
                .map_err(anyhow::Error::from)?;
        }
        return Ok(Action::await_change());
    }

    // Ensure finalizer is set
    ctx.patch(&crd).await?;
    Ok(Action::await_change())
}

pub fn error_policy(crd: Arc<HydraDoomNode>, err: &Error, _ctx: Arc<K8sContext>) -> Action {
    error!(
        error = err.to_string(),
        crd = serde_json::to_string(&crd).unwrap(),
        "reconcile failed"
    );
    Action::requeue(Duration::from_secs(5))
}
