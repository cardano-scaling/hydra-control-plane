use anyhow::Result;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::config::Region;
use futures::StreamExt;
use kube::{runtime::controller::Controller, Api, Client};
use std::sync::Arc;
use tracing::{error, info, instrument};

use hydra_control_plane_operator::{
    config::Config,
    controller::{error_policy, patch_statuses, reconcile, run_autoscaler, K8sContext},
    custom_resource::HydraDoomNode,
};

#[tokio::main]
#[instrument("controller run", skip_all)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Initiating operator.");
    let client = Client::try_default().await?;
    let config = Config::from_env();
    let region_provider = RegionProviderChain::first_try(Region::new(config.bucket_region.clone()));
    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&shared_config);
    let context = Arc::new(K8sContext::new(client.clone(), config, s3_client));

    // Create controller for MyApp custom resource
    let api: Api<HydraDoomNode> = Api::default_namespaced(client);
    info!("Running controller.");
    let controller = Controller::new(api, Default::default())
        .run(reconcile, error_policy, context.clone())
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled {:?}", o),
                Err(e) => error!("Reconcile failed: {:?}", e),
            }
        });
    let patch_statuses_controller = patch_statuses(context.clone());
    let autoscaler_controller = run_autoscaler(context.clone());

    let _ = tokio::join!(controller, patch_statuses_controller, autoscaler_controller);

    Ok(())
}
