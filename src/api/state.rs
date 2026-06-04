use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use crate::domain::{AgentId, FleetCommand, FleetRegistry, JobRecord};

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<dyn FleetRegistry>,
    pub command_queues: Arc<Mutex<HashMap<AgentId, VecDeque<FleetCommand>>>>,
    pub jobs: Arc<Mutex<Vec<JobRecord>>>,
}
