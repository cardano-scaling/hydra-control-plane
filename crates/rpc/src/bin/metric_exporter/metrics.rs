use std::sync::Mutex;

use prometheus::{
    histogram_opts, linear_buckets, Encoder, Histogram, HistogramTimer, IntCounter, IntGauge,
    Registry, TextEncoder,
};

pub enum NodeState {
    Offline,
    Online,
    HeadIsInitializing,
    HeadIsOpen,
}

impl From<NodeState> for i64 {
    fn from(value: NodeState) -> Self {
        match value {
            NodeState::Offline => 0,
            NodeState::Online => 1,
            NodeState::HeadIsInitializing => 2,
            NodeState::HeadIsOpen => 3,
        }
    }
}

pub enum GameState {
    Waiting,
    Lobby,
    Running,
    Done,
}

impl From<GameState> for i64 {
    fn from(value: GameState) -> Self {
        match value {
            GameState::Waiting => 0,
            GameState::Lobby => 1,
            GameState::Running => 2,
            GameState::Done => 3,
        }
    }
}

#[derive(Debug)]
pub struct Metrics {
    pub registry: Registry,
    pub node_state: IntGauge,
    pub game_state: IntGauge,
    pub transactions: IntCounter,
    pub bytes: IntCounter,

    pub games_current: IntGauge,
    pub games_seconds: Histogram,
    pub players_total: IntCounter,
    pub players_current: IntGauge,
    pub kills: IntCounter,
    pub suicides: IntCounter,

    game_timer: Mutex<Option<HistogramTimer>>,
}

impl Metrics {
    pub fn try_new() -> Result<Self, prometheus::Error> {
        let node_state = IntGauge::new(
            "hydra_doom_node_state",
            "0 for OFFLINE, 1 for ONLINE, 2 for HEAD_IS_INITIALIZING, 3 for HEAD_IS_OPEN",
        )
        .unwrap();

        let game_state = IntGauge::new(
            "hydra_doom_game_state",
            "0 for WAITING, 1 for LOBBY, 2 for RUNNING, 3 for DONE",
        )
        .unwrap();

        let transactions = IntCounter::new(
            "hydra_doom_node_transactions",
            "Number of executed transactions.",
        )
        .unwrap();

        let bytes = IntCounter::new(
            "hydra_doom_node_bytes",
            "Number of bytes in executed transactions.",
        )
        .unwrap();

        let games_current = IntGauge::new(
            "hydra_doom_games_current",
            "Number of games currently running.",
        )
        .unwrap();

        let games_seconds = Histogram::with_opts(histogram_opts!(
            "hydra_doom_games_seconds",
            "Duration of games in seconds.",
            linear_buckets(0.0, 60.0, 20)?,
        ))
        .unwrap();

        let players_total = IntCounter::new(
            "hydra_doom_players_total",
            "Total number of players that have joined the game.",
        )
        .unwrap();

        let players_current = IntGauge::new(
            "hydra_doom_players_current",
            "Number of players currently in the game.",
        )
        .unwrap();

        let kills = IntCounter::new("hydra_doom_kills", "Number of kills in the game.").unwrap();

        let suicides =
            IntCounter::new("hydra_doom_suicides", "Number of suicides in the game.").unwrap();

        let registry = Registry::default();
        registry.register(Box::new(node_state.clone()))?;
        registry.register(Box::new(game_state.clone()))?;
        registry.register(Box::new(transactions.clone()))?;
        registry.register(Box::new(bytes.clone()))?;
        registry.register(Box::new(games_current.clone()))?;
        registry.register(Box::new(games_seconds.clone()))?;
        registry.register(Box::new(players_total.clone()))?;
        registry.register(Box::new(players_current.clone()))?;
        registry.register(Box::new(kills.clone()))?;
        registry.register(Box::new(suicides.clone()))?;

        Ok(Self {
            registry,
            node_state,
            game_state,
            transactions,
            bytes,
            games_current,
            games_seconds,
            players_total,
            players_current,
            kills,
            suicides,

            game_timer: Mutex::new(None),
        })
    }

    pub fn set_node_state(&self, state: NodeState) {
        self.node_state.set(state.into())
    }

    pub fn new_transaction(&self, bytes: u64) {
        self.transactions.inc();
        self.bytes.inc_by(bytes);
    }

    pub fn start_server(&self) {
        self.game_state.set(GameState::Waiting.into());
        self.players_current.set(0);
    }

    pub fn server_unavailable(&self) {
        self.game_state.set(GameState::Done.into());
        self.games_current.set(0);
        self.players_current.set(0);
    }

    pub fn start_game(&self) {
        self.games_current.set(1);
        self.game_state.set(GameState::Running.into());
        let mut guard = self.game_timer.lock().unwrap();
        if let Some(prev) = guard.take() {
            // The previous game didn't end properly, so we discard the duration so as not to pollute the timing
            prev.stop_and_discard();
        }
        *guard = Some(self.games_seconds.start_timer());
    }

    pub fn end_game(&self) {
        self.players_current.set(0);
        self.games_current.set(0);
        self.game_state.set(GameState::Done.into());
        let mut guard = self.game_timer.lock().unwrap();
        if let Some(timer) = guard.take() {
            timer.observe_duration();
        }
    }

    pub fn player_joined(&self) {
        self.players_total.inc();
        self.players_current.inc();
        self.game_state.set(GameState::Lobby.into());
    }

    pub fn player_left(&self) {
        self.players_current.dec();
    }

    pub fn player_killed(&self) {
        self.kills.inc();
    }

    pub fn player_suicided(&self) {
        self.suicides.inc();
    }

    pub fn gather(&self) -> String {
        // Encode the metrics in a format that Prometheus can read
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder
            .encode(&self.registry.gather(), &mut buffer)
            .unwrap();

        // Return the metrics as a UTF-8 string
        String::from_utf8(buffer).unwrap()
    }
}
