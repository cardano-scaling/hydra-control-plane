use std::sync::Arc;

use tokio::sync::RwLock;

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
    pub fn from_nodes(nodes: Vec<Node>) -> Self {
        Self {
            state: Arc::new(RwLock::new(InternalState { nodes })),
        }
    }
}
