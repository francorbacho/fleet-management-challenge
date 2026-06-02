use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct UnitId(Uuid);

impl UnitId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UnitId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UnitId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for UnitId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(value)?))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FleetUnit {
    pub id: UnitId,
    pub name: String,
}

impl FleetUnit {
    pub fn register(input: NewFleetUnit) -> Self {
        Self {
            id: UnitId::new(),
            name: input.name,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewFleetUnit {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FleetCommand {
    pub id: Uuid,
    pub unit_id: UnitId,
    pub kind: FleetCommandKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work: Option<WorkAssignment>,
}

impl FleetCommand {
    pub fn new(unit_id: UnitId, kind: FleetCommandKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            unit_id,
            kind,
            work: None,
        }
    }

    pub fn do_work(unit_id: UnitId, work: WorkAssignment) -> Self {
        Self {
            id: Uuid::new_v4(),
            unit_id,
            kind: FleetCommandKind::DoWork,
            work: Some(work),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommandRequest {
    pub kind: FleetCommandKind,
    pub work: Option<WorkRequest>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetCommandKind {
    Diagnostics,
    Restart,
    DoWork,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorkRequest {
    pub number: f64,
    pub calculation: WorkCalculation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorkAssignment {
    pub number: f64,
    pub calculation: WorkCalculation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkCalculation {
    Double,
    Square,
    SquareRoot,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorkSubmission {
    pub result: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JobRecord {
    pub job_id: Uuid,
    pub unit_id: UnitId,
    pub number: f64,
    pub calculation: WorkCalculation,
    pub status: JobStatus,
    pub result: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Completed,
}

pub trait FleetDirectory: Send + Sync {
    fn list_units(&self) -> Vec<FleetUnit>;
    fn get_unit(&self, id: UnitId) -> Option<FleetUnit>;
}

pub trait FleetRegistry: FleetDirectory {
    fn register_unit(&self, input: NewFleetUnit) -> Result<FleetUnit, RegistryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("fleet unit not found: {0}")]
    UnitNotFound(UnitId),
}
