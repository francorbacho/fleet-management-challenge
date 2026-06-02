use std::collections::HashMap;
use std::sync::RwLock;

use crate::domain::{
    FleetDirectory, FleetRegistry, FleetUnit, NewFleetUnit, RegistryError, UnitId,
};

#[derive(Default)]
pub struct InMemoryFleetRegistry {
    units: RwLock<HashMap<UnitId, FleetUnit>>,
}

impl FleetDirectory for InMemoryFleetRegistry {
    fn list_units(&self) -> Vec<FleetUnit> {
        let units = self.units.read().expect("fleet registry lock poisoned");
        units.values().cloned().collect()
    }

    fn get_unit(&self, id: UnitId) -> Option<FleetUnit> {
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
}
