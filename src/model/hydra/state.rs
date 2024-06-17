use std::sync::{Arc, RwLock};

use crate::model::node::Node;

#[derive(Default)]
pub struct InternalState {
    pub nodes: Vec<Node>,
}
#[derive(Clone)]
pub struct HydraNodesState {
    pub state: Arc<RwLock<InternalState>>,
}

impl HydraNodesState {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(InternalState::default())),
        }
    }

    pub fn from_nodes(nodes: Vec<Node>) -> Self {
        Self {
            state: Arc::new(RwLock::new(InternalState { nodes })),
        }
    }
}
