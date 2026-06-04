use serde::{Deserialize, Serialize};

use super::AgentId;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FleetUnit {
    pub id: AgentId,
    pub name: String,
}

impl FleetUnit {
    pub fn register(input: NewFleetUnit) -> Self {
        Self {
            id: AgentId::new(),
            name: input.name,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewFleetUnit {
    pub name: String,
}
