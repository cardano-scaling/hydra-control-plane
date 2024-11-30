use anyhow::{Context, Result};
use model::cluster::ClusterState;
use pallas::ledger::addresses::Network;
use rocket::{http::Method, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use routes::{
    add_player::add_player,
    head::head,
    heads::heads,
    health::health,
    new_game::new_game,
    sample_transactions::sample_transactions,
    stats::{global_stats, refresh_stats, StatsState},
};
use serde::Deserialize;
use std::env;
use tracing::error;

mod model;
mod providers;
mod routes;

#[derive(Deserialize)]
pub struct Config {
    pub admin_key_file: String,
    pub remote: bool,
}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment.extract::<Config>().context("invalid config")?;
    let network: Network = env::var("NETWORK_ID")
        .map(|network_str| {
            network_str
                .parse::<u8>()
                .inspect_err(|_| error!("Invalid NETWORK_ID value, defaulting to 0"))
                .unwrap_or_default()
        })
        .inspect_err(|_| error!("Missing NETWORK_ID env var, defaulting to zero"))
        .unwrap_or_default()
        .into();
    // This will start a reflector (aka: local cache) of the cluster state. The `try_default`
    // initializer assumes that this process is running within the cluster or that the local kubeconfig
    // context is set to the cluster. If you wanted to connect to a remote cluster, you can use the
    // `ClusterState::remote` initializer.
    let cluster = ClusterState::try_new(&config.admin_key_file, config.remote, network).await?;
    let stats = StatsState::new(
        refresh_stats()
            .await
            .expect("failed to fetch initial stats"),
    );

    let bg_stats = stats.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let new_stats = refresh_stats().await;
            match new_stats {
                Ok(stats) => bg_stats.update(stats),
                Err(err) => {
                    println!("Failed to fetch stats: {:?}", err);
                },
            }
        }
    });

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
        .manage(stats)
        .mount(
            "/",
            routes![
                new_game,
                heads,
                head,
                add_player,
                sample_transactions,
                global_stats,
                health,
            ],
        )
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;

    Ok(())
}
