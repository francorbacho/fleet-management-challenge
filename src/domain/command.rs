use serde::{Deserialize, Serialize};

use super::{AgentId, random_id};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FleetCommand {
    pub job_id: u64,
    pub agent_id: AgentId,
    pub kind: FleetCommandKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute: Option<ComputeAssignment>,
}

impl FleetCommand {
    pub fn new(agent_id: AgentId, kind: FleetCommandKind) -> Self {
        Self {
            job_id: random_id(),
            agent_id,
            kind,
            compute: None,
        }
    }

    pub fn compute(agent_id: AgentId, compute: ComputeAssignment) -> Self {
        Self {
            job_id: random_id(),
            agent_id,
            kind: FleetCommandKind::Compute,
            compute: Some(compute),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommandRequest {
    pub kind: FleetCommandKind,
    pub compute: Option<ComputeRequest>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetCommandKind {
    Diagnostics,
    Restart,
    Compute,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComputeRequest {
    pub number: f64,
    pub calculation: ComputeCalculation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComputeAssignment {
    pub number: f64,
    pub calculation: ComputeCalculation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputeCalculation {
    Double,
    Square,
    SquareRoot,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComputeSubmission {
    pub result: f64,
}
