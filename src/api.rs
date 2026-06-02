use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tracing::{info, instrument};

use crate::domain::{FleetRegistry, FleetUnit, NewFleetUnit, RegistryError, UnitId};

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<dyn FleetRegistry>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/fleet", get(list_units).post(register_unit))
        .route("/fleet/{unit_id}", get(get_unit))
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
