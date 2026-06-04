use fleet_management_challenge::domain::{
    ComputeAssignment, ComputeCalculation, ComputeSubmission, FleetCommand, FleetCommandKind,
    FleetUnit, NewFleetUnit, display_agent_id, display_job_id, format_job_id,
};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let api_url =
        std::env::var("FLEET_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_owned());
    let agent_name = std::env::var("FLEET_AGENT_NAME").unwrap_or_else(|_| "fleet-agent".to_owned());
    let registration_url = format!("{}/fleet", api_url.trim_end_matches('/'));
    let client = reqwest::Client::new();

    let registered_agent = client
        .post(&registration_url)
        .json(&NewFleetUnit { name: agent_name })
        .send()
        .await?
        .error_for_status()?
        .json::<FleetUnit>()
        .await?;

    info!(
        agent_id = %display_agent_id(registered_agent.id),
        agent_name = %registered_agent.name,
        "agent registered"
    );

    let command_url = format!(
        "{}/fleet/{}/commands/next",
        api_url.trim_end_matches('/'),
        registered_agent.id
    );

    loop {
        match poll_next_command(&client, &command_url).await {
            Ok(Some(command)) => {
                handle_command(&client, api_url.trim_end_matches('/'), command).await
            }
            Ok(None) => {}
            Err(error) => {
                warn!(%error, "command poll failed");
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "info".to_owned());
    let filter = EnvFilter::new(filter);

    fmt().with_env_filter(filter).with_target(false).init();
}

async fn poll_next_command(
    client: &reqwest::Client,
    command_url: &str,
) -> Result<Option<FleetCommand>, reqwest::Error> {
    let response = client.get(command_url).send().await?.error_for_status()?;

    if response.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }

    response.json::<FleetCommand>().await.map(Some)
}

async fn handle_command(client: &reqwest::Client, api_url: &str, command: FleetCommand) {
    match command.kind {
        FleetCommandKind::Diagnostics => {
            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                "running diagnostics"
            );
        }
        FleetCommandKind::Restart => {
            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                "simulating restart"
            );
        }
        FleetCommandKind::Compute => {
            let Some(ref assignment) = command.compute else {
                warn!(
                    job_id = %display_job_id(command.job_id),
                    "compute command missing assignment"
                );
                return;
            };
            let result = calculate(assignment);

            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                number = assignment.number,
                calculation = ?assignment.calculation,
                result = result,
                "completed assigned compute job"
            );

            sleep(Duration::from_secs(5)).await;

            if let Err(error) = submit_compute_result(client, api_url, &command, result).await {
                warn!(
                    %error,
                    job_id = %display_job_id(command.job_id),
                    "failed to submit compute result"
                );
            }
        }
    }
}

fn calculate(assignment: &ComputeAssignment) -> f64 {
    match assignment.calculation {
        ComputeCalculation::Double => assignment.number * 2.0,
        ComputeCalculation::Square => assignment.number * assignment.number,
        ComputeCalculation::SquareRoot => assignment.number.sqrt(),
    }
}

async fn submit_compute_result(
    client: &reqwest::Client,
    api_url: &str,
    command: &FleetCommand,
    result: f64,
) -> Result<(), reqwest::Error> {
    let submit_url = format!(
        "{}/fleet/{}/jobs/{}/submit",
        api_url, command.agent_id, format_job_id(command.job_id)
    );

    client
        .post(submit_url)
        .json(&ComputeSubmission { result })
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
