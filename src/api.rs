use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tokio::time::{Duration, sleep};
use tracing::{info, instrument};

use crate::domain::{
    CommandRequest, FleetCommand, FleetCommandKind, FleetRegistry, FleetUnit, NewFleetUnit,
    RegistryError, UnitId, WorkAssignment, WorkSubmission,
};

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<dyn FleetRegistry>,
    pub command_queues: Arc<Mutex<HashMap<UnitId, VecDeque<FleetCommand>>>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/fleet", get(list_units).post(register_unit))
        .route("/fleet/{unit_id}", get(get_unit))
        .route("/fleet/{unit_id}/commands", post(queue_command))
        .route("/fleet/{unit_id}/commands/next", get(next_command))
        .route("/fleet/{unit_id}/jobs/{job_id}/submit", post(submit_job))
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
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

#[instrument(skip(state, request))]
async fn queue_command(
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

    info!(
        unit_id = %unit_id,
        command_id = %command.id,
        command_kind = ?command.kind,
        "queued fleet command"
    );

    Ok((StatusCode::CREATED, Json(command)))
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
                command_kind = ?command.kind,
                "delivered fleet command"
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
        FleetCommandKind::DoWork => {
            let work = request.work.ok_or(ApiError::BadRequest(
                "do_work commands require a work payload",
            ))?;

            Ok(FleetCommand::do_work(
                unit_id,
                WorkAssignment {
                    number: work.number,
                    calculation: work.calculation,
                },
            ))
        }
    }
}

#[instrument(skip(state, submission))]
async fn submit_job(
    State(state): State<AppState>,
    Path((unit_id, job_id)): Path<(UnitId, uuid::Uuid)>,
    Json(submission): Json<WorkSubmission>,
) -> Result<StatusCode, ApiError> {
    state.registry.get_unit(unit_id).ok_or(ApiError::NotFound)?;

    info!(
        unit_id = %unit_id,
        job_id = %job_id,
        result = submission.result,
        "received completed job submission"
    );

    Ok(StatusCode::ACCEPTED)
}

#[derive(Debug)]
enum ApiError {
    NotFound,
    BadRequest(&'static str),
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
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

const INDEX_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Fleet Control</title>
  <style>
    :root { color-scheme: light; font-family: system-ui, sans-serif; background: #f6f7f9; color: #1b1f24; }
    body { margin: 0; }
    main { max-width: 980px; margin: 0 auto; padding: 32px 20px; }
    header { display: flex; align-items: baseline; justify-content: space-between; gap: 16px; margin-bottom: 24px; }
    h1 { font-size: 28px; margin: 0; letter-spacing: 0; }
    button, input, select { font: inherit; }
    button { border: 1px solid #b8c0cc; background: #fff; border-radius: 6px; padding: 8px 10px; cursor: pointer; }
    button.primary { background: #175cd3; border-color: #175cd3; color: #fff; }
    button:hover { filter: brightness(0.97); }
    .agents { display: grid; gap: 12px; }
    .agent { background: #fff; border: 1px solid #d8dee8; border-radius: 8px; padding: 14px; }
    .agent-head { display: flex; justify-content: space-between; gap: 12px; margin-bottom: 12px; }
    .name { font-weight: 700; }
    .id { color: #667085; font-size: 13px; overflow-wrap: anywhere; }
    .actions { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
    .work { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; padding-top: 8px; border-top: 1px solid #edf0f5; margin-top: 8px; }
    input[type="number"] { width: 110px; border: 1px solid #b8c0cc; border-radius: 6px; padding: 8px; }
    select { border: 1px solid #b8c0cc; border-radius: 6px; padding: 8px; background: #fff; }
    .empty { color: #667085; padding: 24px 0; }
    .status { min-height: 22px; color: #344054; }
  </style>
</head>
<body>
  <main>
    <header>
      <h1>Fleet Control</h1>
      <button id="refresh">Refresh</button>
    </header>
    <p class="status" id="status"></p>
    <section class="agents" id="agents"></section>
  </main>
  <script>
    const agentsEl = document.querySelector("#agents");
    const statusEl = document.querySelector("#status");

    document.querySelector("#refresh").addEventListener("click", loadAgents);
    loadAgents();
    setInterval(loadAgents, 5000);

    async function loadAgents() {
      const agents = await fetchJson("/fleet");
      agentsEl.innerHTML = "";
      if (agents.length === 0) {
        agentsEl.innerHTML = '<div class="empty">No agents connected.</div>';
        return;
      }
      for (const agent of agents) agentsEl.appendChild(renderAgent(agent));
    }

    function renderAgent(agent) {
      const el = document.createElement("article");
      el.className = "agent";
      el.innerHTML = `
        <div class="agent-head">
          <div>
            <div class="name">${escapeHtml(agent.name)}</div>
            <div class="id">${agent.id}</div>
          </div>
        </div>
        <div class="actions">
          <button data-kind="diagnostics">Diagnostics</button>
          <button data-kind="restart">Restart</button>
        </div>
        <div class="work">
          <input type="number" step="any" value="12.5" aria-label="Work number">
          <select aria-label="Calculation">
            <option value="double">Double</option>
            <option value="square">Square</option>
            <option value="square_root">Square root</option>
          </select>
          <button class="primary" data-kind="do_work">Do work</button>
        </div>
      `;

      el.querySelectorAll("button[data-kind]").forEach(button => {
        button.addEventListener("click", () => queueCommand(agent.id, button.dataset.kind, el));
      });
      return el;
    }

    async function queueCommand(agentId, kind, root) {
      const body = { kind };
      if (kind === "do_work") {
        body.work = {
          number: Number(root.querySelector("input").value),
          calculation: root.querySelector("select").value
        };
      }

      const command = await fetchJson(`/fleet/${agentId}/commands`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body)
      });
      statusEl.textContent = `Queued ${command.kind} for ${agentId}`;
    }

    async function fetchJson(url, options) {
      const response = await fetch(url, options);
      if (!response.ok) throw new Error(await response.text());
      return response.json();
    }

    function escapeHtml(value) {
      return value.replace(/[&<>"']/g, char => ({
        "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"
      }[char]));
    }
  </script>
</body>
</html>"##;
