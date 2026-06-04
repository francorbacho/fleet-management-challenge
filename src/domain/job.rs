use serde::{Deserialize, Serialize};

use super::{AgentId, ComputeCalculation};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JobRecord {
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
    Completed,
}
