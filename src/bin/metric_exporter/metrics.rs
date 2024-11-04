use prometheus::{Encoder, IntCounter, IntGauge, Registry, TextEncoder};

pub enum NodeState {
    Offline,
    Online,
    HeadIsInitializing,
    HeadIsOpen,
}

#[derive(Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub state: IntGauge,
    pub transactions: IntCounter,
    // pub kills: IntCounterVec,
    // pub items: IntCounterVec,
    // pub secrets: IntCounterVec,
}

impl Metrics {
    pub fn try_new() -> Result<Self, prometheus::Error> {
        let state = IntGauge::new(
            "hydra_doom_node_state",
            "0 for OFFLINE, 1 for ONLINE, 2 for HEAD_IS_INITIALIZING, 3 for HEAD_IS_OPEN",
        )
        .unwrap();

        let transactions = IntCounter::new(
            "hydra_doom_node_transactions",
            "Number of executed transactions.",
        )
        .unwrap();

        let registry = Registry::default();
        registry.register(Box::new(state.clone()))?;
        registry.register(Box::new(transactions.clone()))?;

        Ok(Self {
            registry,
            state,
            transactions,
        })
    }

    pub fn set_state(&self, state: NodeState) {
        self.state.set(match state {
            NodeState::Offline => 0,
            NodeState::Online => 1,
            NodeState::HeadIsInitializing => 2,
            NodeState::HeadIsOpen => 3,
        })
    }

    pub fn inc_transactions(&self) {
        self.transactions.inc()
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
