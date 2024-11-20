use rocket::{get, serde::json::Json};
use rocket_errors::anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct GlobalStats {
    total_games: u32,
    active_games: u32,
    total_players: u32,
    active_player: u32,
    total_bots: u32,
    active_bots: u32,
    total_txs: u32,
    txs_per_second: u32,
    total_bytes: u32,
    bytes_per_second: u32,
    total_kills: u32,
    kills_per_minute: u32,
    total_suicides: u32,
    suicides_per_minute: u32,
}

#[get("/global_stats")]
pub async fn global_stats() -> Result<Json<GlobalStats>> {
    // TODO: fetch data from the observer server
    // dummy data for now
    Ok(Json(GlobalStats{
        total_games: 123,
        active_games: 21,
        total_players: 125,
        active_player: 21,
        total_bots: 321,
        active_bots: 21,
        total_txs: 123456,
        txs_per_second: 5000,
        total_bytes: 123456789,
        bytes_per_second: 12000,
        total_kills: 100,
        kills_per_minute: 30,
        total_suicides: 1,
        suicides_per_minute: 0,
    }))
}
