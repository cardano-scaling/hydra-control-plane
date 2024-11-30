use futures_util::try_join;
use rocket::{get, serde::json::Json, State};
use rocket_errors::anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::{Arc, RwLock}, time};

#[derive(Clone)]
pub struct StatsState {
    latest_stats: Arc<RwLock<GlobalStats>>,
}

impl StatsState {
    pub fn new(stats: GlobalStats) -> Self {
        Self {
            latest_stats: Arc::new(RwLock::new(stats)),
        }
    }

    pub fn update(&self, stats: GlobalStats) {
        let mut latest_stats = self.latest_stats.write().unwrap();
        *latest_stats = stats;
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct GlobalStats {
    as_of: time::SystemTime,
    total_txs: u64,
    txs_per_second: f64,
    peak_txs_per_second: f64,
    total_bytes: u64,
    bytes_per_second: f64,

    total_games: u32,
    active_games: u32,
    total_players: u32,
    active_players: u32,
    total_bots: u32,
    active_bots: u32,
    total_kills: u32,
    kills_per_minute: f32,
    total_suicides: u32,
    suicides_per_minute: f32,
}

const TOTAL_TRANSACTIONS: &str = "sum(last_over_time(hydra_doom_node_transactions[1y]))";
const TRANSACTIONS_PER_SECOND: &str = "sum(irate(hydra_doom_node_transactions[1m])>0)";
const PEAK_TRANSACTIONS_PER_SECOND: &str = "max_over_time(sum(irate(hydra_doom_node_transactions[1m]))[1w:])";
const TOTAL_BYTES: &str = "sum(last_over_time(hydra_doom_node_bytes[1y]))";
const BYTES_PER_SECOND: &str = "sum(irate(hydra_doom_node_bytes[1m])>0)";
const TOTAL_GAMES: &str = "sum(last_over_time(hydra_doom_games_seconds_count[1y]))";
const ACTIVE_GAMES: &str = "sum(hydra_doom_games_current)";
const TOTAL_PLAYERS: &str = "sum(last_over_time(hydra_doom_players_total[1y]))";
const ACTIVE_PLAYERS: &str = "sum(hydra_doom_players_current)";
const TOTAL_BOTS: &str = "sum(last_over_time(hydra_doom_bots_total[1y]))";
const ACTIVE_BOTS: &str = "";
const TOTAL_KILLS: &str = "sum(last_over_time(hydra_doom_kills[1y]))";
const KILLS_PER_MINUTE: &str = "sum(irate(hydra_doom_kills[10m]) * 60)";

#[derive(Deserialize, Debug)]
struct ThanosResult {
    pub value: (f32, String),
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ThanosData {
    #[expect(dead_code)]
    pub result_type: String,
    pub result: Vec<ThanosResult>,
}
#[derive(Deserialize, Debug)]
struct ThanosResponse {
    pub status: String,
    #[expect(dead_code)]
    pub data: ThanosData,
}
pub async fn fetch_metric<T: FromStr + Default>(query: &str) -> Result<T> {
    let url = format!(
        "https://thanos.hydra-doom.sundae.fi/api/v1/query?query={}",
        query
    );
    let resp = reqwest::get(&url).await?;
    // println!("{:?}", resp.text().await?);
    let body = resp.json::<ThanosResponse>().await?;
    if body.data.result.len() == 0 {
        Ok(Default::default())
    } else {
        let parsed = body.data.result[0].value.1.parse::<T>();
        match parsed {
            Ok(v) => Ok(v),
            Err(_) => {
                println!(
                    "Invalid stats value for {}: {}",
                    query, body.data.result[0].value.1
                );
                Ok(Default::default())
            }
        }
    }
}

pub async fn refresh_stats() -> Result<GlobalStats> {
    let (
        total_txs,
        txs_per_second,
        peak_txs_per_second,
        total_bytes,
        bytes_per_second,
        total_games,
        active_games,
        total_players,
        active_players,
        total_kills,
        kills_per_minute,
    ) = try_join!(
        fetch_metric(TOTAL_TRANSACTIONS),
        fetch_metric(TRANSACTIONS_PER_SECOND),
        fetch_metric(PEAK_TRANSACTIONS_PER_SECOND),
        fetch_metric(TOTAL_BYTES),
        fetch_metric(BYTES_PER_SECOND),
        fetch_metric(TOTAL_GAMES),
        fetch_metric(ACTIVE_GAMES),
        fetch_metric(TOTAL_PLAYERS),
        fetch_metric(ACTIVE_PLAYERS),
        fetch_metric(TOTAL_KILLS),
        fetch_metric(KILLS_PER_MINUTE),
    )?;
    Ok(GlobalStats {
        as_of: time::SystemTime::now(),
        total_txs,
        txs_per_second,
        peak_txs_per_second,
        total_bytes,
        bytes_per_second,
        total_games,
        active_games,
        total_players,
        active_players,
        total_bots: 0,
        active_bots: 0,
        total_kills,
        kills_per_minute,
        total_suicides: 0,
        suicides_per_minute: 0.0,
    })
}

#[get("/global_stats")]
pub async fn global_stats(state: &State<StatsState>) -> Result<Json<GlobalStats>> {
    let stats = state.latest_stats.read().unwrap();
    Ok(Json(stats.clone()))
}
