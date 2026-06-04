use std::collections::VecDeque;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};

use super::error::ApiError;
use super::state::AppState;
use crate::domain::{
    AgentId, CommandRequest, ComputeAssignment, FleetCommand, FleetCommandKind,
    FleetUnit, JobRecord, JobStatus, JobSubmission, NewFleetUnit, display_agent_id, display_job_id,
    parse_job_id,
};

pub(super) async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

pub(super) async fn list_units(State(state): State<AppState>) -> Json<Vec<FleetUnit>> {
    Json(state.registry.list_units())
}

pub(super) async fn list_jobs(State(state): State<AppState>) -> Json<Vec<JobRecord>> {
    let mut jobs = state.jobs.lock().expect("job table lock poisoned").clone();
    jobs.sort_by_key(|job| job.job_id);

    Json(jobs)
}

pub(super) async fn register_unit(
    State(state): State<AppState>,
    Json(input): Json<NewFleetUnit>,
) -> Result<(StatusCode, Json<FleetUnit>), ApiError> {
    let unit = state.registry.register_unit(input)?;

    info!(agent_id = %display_agent_id(unit.id), "registered agent");

    Ok((StatusCode::CREATED, Json(unit)))
}

pub(super) async fn queue_command(
    State(state): State<AppState>,
    Path(agent_id): Path<AgentId>,
    Json(request): Json<CommandRequest>,
) -> Result<(StatusCode, Json<FleetCommand>), ApiError> {
    state
        .registry
        .get_unit(agent_id)
        .ok_or(ApiError::NotFound)?;

    let command = build_command(agent_id, request)?;
    {
        let mut queues = state
            .command_queues
            .lock()
            .expect("command queue lock poisoned");
        queues
            .entry(agent_id)
            .or_default()
            .push_back(command.clone());
    }
    track_job(&state, &command);

    info!(
        agent_id = %display_agent_id(agent_id),
        job_id = %display_job_id(command.job_id),
        kind = ?command.kind,
        "queued command"
    );

    Ok((StatusCode::CREATED, Json(command)))
}

pub(super) async fn get_unit(
    State(state): State<AppState>,
    Path(agent_id): Path<AgentId>,
) -> Result<Json<FleetUnit>, ApiError> {
    state
        .registry
        .get_unit(agent_id)
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(super) async fn next_command(
    State(state): State<AppState>,
    Path(agent_id): Path<AgentId>,
) -> Result<Response, ApiError> {
    state
        .registry
        .get_unit(agent_id)
        .ok_or(ApiError::NotFound)?;

    state.registry.set_connected(agent_id);
    let guard = PollGuard {
        state: state.clone(),
        agent_id,
    };

    let mut waited = Duration::ZERO;
    let interval = Duration::from_millis(250);
    let timeout = Duration::from_secs(30);

    while waited < timeout {
        if let Some(command) = pop_next_command(&state, agent_id) {
            accept_job(&state, &command);

            info!(
                agent_id = %display_agent_id(agent_id),
                job_id = %display_job_id(command.job_id),
                kind = ?command.kind,
                "delivered command"
            );

            // Agent received a command and will poll again — keep it connected
            guard.disarm();
            return Ok(Json(command).into_response());
        }

        sleep(interval).await;
        waited += interval;
    }

    // Normal timeout — agent will immediately re-poll, keep it connected
    guard.disarm();
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Marks an agent as disconnected when dropped without being disarmed.
/// If the handler future is cancelled (client disconnected), the drop
/// runs and marks the agent offline instantly.
struct PollGuard {
    state: AppState,
    agent_id: AgentId,
}

impl PollGuard {
    fn disarm(self) {
        std::mem::forget(self);
    }
}

impl Drop for PollGuard {
    fn drop(&mut self) {
        warn!(
            agent_id = %display_agent_id(self.agent_id),
            "agent disconnected"
        );
        self.state.registry.mark_disconnected(self.agent_id);
    }
}

fn pop_next_command(state: &AppState, agent_id: AgentId) -> Option<FleetCommand> {
    let mut queues = state
        .command_queues
        .lock()
        .expect("command queue lock poisoned");
    queues.get_mut(&agent_id).and_then(VecDeque::pop_front)
}

fn build_command(agent_id: AgentId, request: CommandRequest) -> Result<FleetCommand, ApiError> {
    match request.kind {
        FleetCommandKind::Diagnostics => {
            Ok(FleetCommand::new(agent_id, FleetCommandKind::Diagnostics))
        }
        FleetCommandKind::Restart => Ok(FleetCommand::new(agent_id, FleetCommandKind::Restart)),
        FleetCommandKind::Compute => {
            let compute = request.compute.ok_or(ApiError::BadRequest(
                "compute commands require a compute payload",
            ))?;

            Ok(FleetCommand::compute(
                agent_id,
                ComputeAssignment {
                    number: compute.number,
                    calculation: compute.calculation,
                },
            ))
        }
    }
}

fn track_job(state: &AppState, command: &FleetCommand) {
    let Some(compute) = command.compute.clone() else {
        return;
    };

    let job = JobRecord {
        job_id: command.job_id,
        agent_id: command.agent_id,
        number: compute.number,
        calculation: compute.calculation,
        status: JobStatus::Pending,
        result: None,
    };

    state
        .jobs
        .lock()
        .expect("job table lock poisoned")
        .push(job);
}

fn accept_job(state: &AppState, command: &FleetCommand) {
    if command.compute.is_none() {
        return;
    }

    let mut jobs = state.jobs.lock().expect("job table lock poisoned");
    if let Some(job) = jobs.iter_mut().find(|job| job.job_id == command.job_id) {
        job.status = JobStatus::Accepted;
    }
}

pub(super) async fn submit_job(
    State(state): State<AppState>,
    Path((agent_id, job_id_str)): Path<(AgentId, String)>,
    Json(submission): Json<JobSubmission>,
) -> Result<StatusCode, ApiError> {
    state
        .registry
        .get_unit(agent_id)
        .ok_or(ApiError::NotFound)?;
    let job_id = parse_job_id(&job_id_str).map_err(|_| ApiError::BadRequest("invalid job id"))?;
    complete_job(&state, agent_id, job_id, &submission)?;

    info!(
        agent_id = %display_agent_id(agent_id),
        job_id = %display_job_id(job_id),
        result = submission.result,
        "succeeded job"
    );

    Ok(StatusCode::ACCEPTED)
}

fn complete_job(
    state: &AppState,
    agent_id: AgentId,
    job_id: u64,
    submission: &JobSubmission,
) -> Result<(), ApiError> {
    let mut jobs = state.jobs.lock().expect("job table lock poisoned");
    let job = jobs
        .iter_mut()
        .find(|job| job.job_id == job_id)
        .ok_or(ApiError::NotFound)?;

    if job.agent_id != agent_id {
        return Err(ApiError::BadRequest(
            "job submission does not match assignment",
        ));
    }

    job.status = JobStatus::Succeed;
    job.result = Some(submission.result);

    Ok(())
}
