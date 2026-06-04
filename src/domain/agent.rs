use serde::{Deserialize, Serialize};

use super::AgentId;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Connected,
    Disconnected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FleetUnit {
    pub id: AgentId,
    pub name: String,
    pub status: AgentStatus,
}

impl FleetUnit {
    pub fn register(input: NewFleetUnit) -> Self {
        Self {
            id: AgentId::new(),
            name: input.name,
            status: AgentStatus::Connected,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewFleetUnit {
    pub name: String,
}
