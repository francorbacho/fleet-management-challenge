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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FleetEvent {
    pub id: Uuid,
    pub unit_id: UnitId,
    pub kind: FleetEventKind,
}

impl FleetEvent {
    pub fn diagnostics(unit_id: UnitId) -> Self {
        Self {
            id: Uuid::new_v4(),
            unit_id,
            kind: FleetEventKind::Diagnostics,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetEventKind {
    Diagnostics,
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
