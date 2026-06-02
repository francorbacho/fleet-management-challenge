use std::collections::VecDeque;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::time::{Duration, sleep};
use tracing::info;

use super::error::ApiError;
use super::state::AppState;
use crate::domain::{
    CommandRequest, ComputeAssignment, ComputeSubmission, FleetCommand, FleetCommandKind,
    FleetUnit, JobRecord, JobStatus, NewFleetUnit, UnitId,
};

pub(super) async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

pub(super) async fn list_units(State(state): State<AppState>) -> Json<Vec<FleetUnit>> {
    Json(state.registry.list_units())
}

pub(super) async fn list_jobs(State(state): State<AppState>) -> Json<Vec<JobRecord>> {
    let mut jobs = state
        .jobs
        .lock()
        .expect("job table lock poisoned")
        .values()
        .cloned()
        .collect::<Vec<_>>();
    jobs.sort_by_key(|job| job.job_id);

    Json(jobs)
}

pub(super) async fn register_unit(
    State(state): State<AppState>,
    Json(input): Json<NewFleetUnit>,
) -> Result<(StatusCode, Json<FleetUnit>), ApiError> {
    let unit = state.registry.register_unit(input)?;

    info!(unit_id = %unit.id, "registered unit");

    Ok((StatusCode::CREATED, Json(unit)))
}

pub(super) async fn queue_command(
    State(state): State<AppState>,
    Path(unit_id): Path<UnitId>,
    Json(request): Json<CommandRequest>,
) -> Result<(StatusCode, Json<FleetCommand>), ApiError> {
    state.registry.get_unit(unit_id).ok_or(ApiError::NotFound)?;

    let command = build_command(unit_id, request)?;
    {
        let mut queues = state
            .command_queues
            .lock()
            .expect("command queue lock poisoned");
        queues
            .entry(unit_id)
            .or_default()
            .push_back(command.clone());
    }
    track_job(&state, &command);

    info!(
        unit_id = %unit_id,
        command_id = %command.id,
        kind = ?command.kind,
        "queued command"
    );

    Ok((StatusCode::CREATED, Json(command)))
}

pub(super) async fn get_unit(
    State(state): State<AppState>,
    Path(unit_id): Path<UnitId>,
) -> Result<Json<FleetUnit>, ApiError> {
    state
        .registry
        .get_unit(unit_id)
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(super) async fn next_command(
    State(state): State<AppState>,
    Path(unit_id): Path<UnitId>,
) -> Result<Response, ApiError> {
    state.registry.get_unit(unit_id).ok_or(ApiError::NotFound)?;

    let mut waited = Duration::ZERO;
    let interval = Duration::from_millis(250);
    let timeout = Duration::from_secs(30);

    while waited < timeout {
        if let Some(command) = pop_next_command(&state, unit_id) {
            info!(
                unit_id = %unit_id,
                command_id = %command.id,
                kind = ?command.kind,
                "delivered command"
            );

            return Ok(Json(command).into_response());
        }

        sleep(interval).await;
        waited += interval;
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

fn pop_next_command(state: &AppState, unit_id: UnitId) -> Option<FleetCommand> {
    let mut queues = state
        .command_queues
        .lock()
        .expect("command queue lock poisoned");
    queues.get_mut(&unit_id).and_then(VecDeque::pop_front)
}

fn build_command(unit_id: UnitId, request: CommandRequest) -> Result<FleetCommand, ApiError> {
    match request.kind {
        FleetCommandKind::Diagnostics => {
            Ok(FleetCommand::new(unit_id, FleetCommandKind::Diagnostics))
        }
        FleetCommandKind::Restart => Ok(FleetCommand::new(unit_id, FleetCommandKind::Restart)),
        FleetCommandKind::Compute => {
            let compute = request.compute.ok_or(ApiError::BadRequest(
                "compute commands require a compute payload",
            ))?;

            Ok(FleetCommand::compute(
                unit_id,
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
        job_id: command.id,
        unit_id: command.unit_id,
        number: compute.number,
        calculation: compute.calculation,
        status: JobStatus::Pending,
        result: None,
    };

    state
        .jobs
        .lock()
        .expect("job table lock poisoned")
        .insert(job.job_id, job);
}

pub(super) async fn submit_job(
    State(state): State<AppState>,
    Path((unit_id, job_id)): Path<(UnitId, uuid::Uuid)>,
    Json(submission): Json<ComputeSubmission>,
) -> Result<StatusCode, ApiError> {
    state.registry.get_unit(unit_id).ok_or(ApiError::NotFound)?;
    complete_job(&state, unit_id, job_id, &submission)?;

    info!(
        unit_id = %unit_id,
        job_id = %job_id,
        result = submission.result,
        "completed job"
    );

    Ok(StatusCode::ACCEPTED)
}

fn complete_job(
    state: &AppState,
    unit_id: UnitId,
    job_id: uuid::Uuid,
    submission: &ComputeSubmission,
) -> Result<(), ApiError> {
    let mut jobs = state.jobs.lock().expect("job table lock poisoned");
    let job = jobs.get_mut(&job_id).ok_or(ApiError::NotFound)?;

    if job.unit_id != unit_id {
        return Err(ApiError::BadRequest(
            "job submission does not match assignment",
        ));
    }

    job.status = JobStatus::Completed;
    job.result = Some(submission.result);

    Ok(())
}
