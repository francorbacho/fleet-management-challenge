use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tokio::time::{Duration, sleep};
use tracing::{info, instrument};

use crate::domain::{
    FleetCommand, FleetCommandKind, FleetRegistry, FleetUnit, NewFleetUnit, RegistryError, UnitId,
};

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<dyn FleetRegistry>,
    pub command_sequence: Arc<AtomicUsize>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/fleet", get(list_units).post(register_unit))
        .route("/fleet/{unit_id}", get(get_unit))
        .route("/fleet/{unit_id}/commands/next", get(next_command))
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[instrument(skip(state))]
async fn list_units(State(state): State<AppState>) -> Json<Vec<FleetUnit>> {
    Json(state.registry.list_units())
}

#[instrument(skip(state, input))]
async fn register_unit(
    State(state): State<AppState>,
    Json(input): Json<NewFleetUnit>,
) -> Result<(StatusCode, Json<FleetUnit>), ApiError> {
    let unit = state.registry.register_unit(input)?;

    info!(unit_id = %unit.id, "registered fleet unit");

    Ok((StatusCode::CREATED, Json(unit)))
}

#[instrument(skip(state))]
async fn get_unit(
    State(state): State<AppState>,
    Path(unit_id): Path<UnitId>,
) -> Result<Json<FleetUnit>, ApiError> {
    state
        .registry
        .get_unit(unit_id)
        .map(Json)
        .ok_or(ApiError::NotFound)
}

#[instrument(skip(state))]
async fn next_command(
    State(state): State<AppState>,
    Path(unit_id): Path<UnitId>,
) -> Result<Json<FleetCommand>, ApiError> {
    state.registry.get_unit(unit_id).ok_or(ApiError::NotFound)?;

    sleep(Duration::from_secs(2)).await;

    let command = FleetCommand::new(unit_id, next_command_kind(&state));

    info!(
        unit_id = %unit_id,
        command_id = %command.id,
        command_kind = ?command.kind,
        "delivered fleet command"
    );

    Ok(Json(command))
}

fn next_command_kind(state: &AppState) -> FleetCommandKind {
    match state.command_sequence.fetch_add(1, Ordering::Relaxed) % 3 {
        0 => FleetCommandKind::Diagnostics,
        1 => FleetCommandKind::Restart,
        _ => FleetCommandKind::DoWork,
    }
}

#[derive(Debug)]
enum ApiError {
    NotFound,
}

impl From<RegistryError> for ApiError {
    fn from(error: RegistryError) -> Self {
        match error {
            RegistryError::UnitNotFound(_) => Self::NotFound,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "fleet unit not found"),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}
