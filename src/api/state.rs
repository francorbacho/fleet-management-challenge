use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use crate::domain::{FleetCommand, FleetRegistry, JobRecord, UnitId};

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<dyn FleetRegistry>,
    pub command_queues: Arc<Mutex<HashMap<UnitId, VecDeque<FleetCommand>>>>,
    pub jobs: Arc<Mutex<HashMap<uuid::Uuid, JobRecord>>>,
}
