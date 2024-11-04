use std::path::PathBuf;

use anyhow::{Context, Result};
use hydra_control_plane::NodeConfig;
use model::cluster::ClusterState;
use rocket::{http::Method, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use routes::{add_player::add_player, head::head, heads::heads, new_game::new_game};
use serde::Deserialize;
use tokio::{
    spawn,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tracing::{info, warn};

mod model;
mod providers;
mod routes;

#[derive(Deserialize)]
pub struct Config {}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment.extract::<Config>().context("invalid config")?;

    // This will start a reflector (aka: local cache) of the cluster state. The `try_default`
    // initializer assumes that this process is running within the cluster or that the local kubeconfig
    // context is set to the cluster. If you wanted to connect to a remote cluster, you can use the
    // `ClusterState::remote` initializer.
    let cluster = ClusterState::try_default().await?;

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let _rocket = rocket::build()
        .manage(cluster)
        .mount("/", routes![new_game, heads, head, add_player])
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;

    Ok(())
}
