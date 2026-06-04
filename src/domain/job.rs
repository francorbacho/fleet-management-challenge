use serde::{Deserialize, Serialize};

use super::{AgentId, ComputeCalculation};

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
pub struct JobRecord {
    #[serde(with = "hex_job_id")]
    pub job_id: u64,
    pub agent_id: AgentId,
    pub number: f64,
    pub calculation: ComputeCalculation,
    pub status: JobStatus,
    pub result: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Accepted,
    Succeed,
    Failed,
}
