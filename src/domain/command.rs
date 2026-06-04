use serde::{Deserialize, Serialize};

use super::{AgentId, random_id};

mod hex_job_id {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(id: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:x}", id))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        let s = String::deserialize(deserializer)?;
        u64::from_str_radix(&s, 16).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FleetCommand {
    #[serde(with = "hex_job_id")]
    pub job_id: u64,
    pub agent_id: AgentId,
    pub request: CommandRequest,
}

impl FleetCommand {
    pub fn new(agent_id: AgentId, request: CommandRequest) -> Self {
        Self {
            job_id: random_id(),
            agent_id,
            request,
        }
    }

    pub fn double(agent_id: AgentId, value: f64) -> Self {
        Self {
            job_id: random_id(),
            agent_id,
            request: CommandRequest::Double(value),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandRequest {
    Diagnostics,
    Restart,
    Double(f64),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JobSubmission {
    pub result: String,
}
