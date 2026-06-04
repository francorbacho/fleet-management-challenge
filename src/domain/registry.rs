use super::{AgentId, FleetUnit, NewFleetUnit};

pub trait FleetDirectory: Send + Sync {
    fn list_units(&self) -> Vec<FleetUnit>;
    fn get_unit(&self, id: AgentId) -> Option<FleetUnit>;
}

pub trait FleetRegistry: FleetDirectory {
    fn register_unit(&self, input: NewFleetUnit) -> Result<FleetUnit, RegistryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("fleet unit not found: {0}")]
    UnitNotFound(AgentId),
}
