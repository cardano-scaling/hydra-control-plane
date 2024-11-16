use rocket::{get, serde::json::Json};
use rocket_errors::anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

//TODO: determine how to store the "time series" data
#[derive(Debug, Serialize)]
pub struct GlobalStats {
    total_games: u32,
    active_games: u32,
    total_players: u32,
    active_player: u32,
    total_bots: u32,
    active_bots: u32,
    total_txs: u32,
    txs_time_series: Vec<u32>,
    total_bytes: u32,
    bytes_time_series: Vec<u32>,
    total_kills: u32,
    kills_time_series: Vec<u32>,
    player_kills: HashMap<String, u32>,
    total_suicides: u32,
    suicides_time_series: Vec<u32>,
    player_suicides: HashMap<String, u32>,
    total_deaths: u32,
    deaths_time_series: Vec<u32>,
    player_deaths: HashMap<String, u32>,
}

#[get("/global_stats")]
pub async fn global_stats() -> Result<Json<GlobalStats>> {
    // fetch data from the observer server
    todo!()
}
