use axum::Router;
use axum::routing::{get, post};

mod error;
mod handlers;
mod registry;
mod state;
mod web;

pub use registry::InMemoryFleetRegistry;
pub use state::AppState;

use handlers::{
    get_unit, health, list_jobs, list_units, next_command, queue_command, register_unit, submit_job,
};
use web::index;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/fleet", get(list_units).post(register_unit))
        .route("/jobs", get(list_jobs))
        .route("/fleet/{agent_id}", get(get_unit))
        .route("/fleet/{agent_id}/commands", post(queue_command))
        .route("/fleet/{agent_id}/commands/next", get(next_command))
        .route("/fleet/{agent_id}/jobs/{job_id}/submit", post(submit_job))
        .with_state(state)
}
