use std::collections::HashMap;
use std::sync::RwLock;

use crate::domain::{
    AgentId, AgentStatus, FleetDirectory, FleetRegistry, FleetUnit, NewFleetUnit, RegistryError,
};

#[derive(Default)]
pub struct InMemoryFleetRegistry {
    units: RwLock<HashMap<AgentId, FleetUnit>>,
}

impl FleetDirectory for InMemoryFleetRegistry {
    fn list_units(&self) -> Vec<FleetUnit> {
        let units = self.units.read().expect("fleet registry lock poisoned");
        units.values().cloned().collect()
    }

    fn get_unit(&self, id: AgentId) -> Option<FleetUnit> {
        let units = self.units.read().expect("fleet registry lock poisoned");
        units.get(&id).cloned()
    }
}

impl FleetRegistry for InMemoryFleetRegistry {
    fn register_unit(&self, input: NewFleetUnit) -> Result<FleetUnit, RegistryError> {
        let unit = FleetUnit::register(input);
        let mut units = self.units.write().expect("fleet registry lock poisoned");
        units.insert(unit.id, unit.clone());

        Ok(unit)
    }

    fn set_connected(&self, id: AgentId) {
        let mut units = self.units.write().expect("fleet registry lock poisoned");
        if let Some(unit) = units.get_mut(&id) {
            unit.status = AgentStatus::Connected;
        }
    }

    fn mark_disconnected(&self, id: AgentId) {
        let mut units = self.units.write().expect("fleet registry lock poisoned");
        if let Some(unit) = units.get_mut(&id) {
            unit.status = AgentStatus::Disconnected;
        }
    }
}
