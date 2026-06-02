use fleet_management_challenge::domain::{FleetCommand, FleetCommandKind, FleetUnit, NewFleetUnit};
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

    let registered_unit = client
        .post(&registration_url)
        .json(&NewFleetUnit { name: agent_name })
        .send()
        .await?
        .error_for_status()?
        .json::<FleetUnit>()
        .await?;

    info!(
        unit_id = %registered_unit.id,
        unit_name = %registered_unit.name,
        "agent registered"
    );

    let command_url = format!(
        "{}/fleet/{}/commands/next",
        api_url.trim_end_matches('/'),
        registered_unit.id
    );

    loop {
        match poll_next_command(&client, &command_url).await {
            Ok(command) => handle_command(command).await,
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
) -> Result<FleetCommand, reqwest::Error> {
    client
        .get(command_url)
        .send()
        .await?
        .error_for_status()?
        .json::<FleetCommand>()
        .await
}

async fn handle_command(command: FleetCommand) {
    match command.kind {
        FleetCommandKind::Diagnostics => {
            info!(
                command_id = %command.id,
                unit_id = %command.unit_id,
                "running diagnostics"
            );
        }
        FleetCommandKind::Restart => {
            info!(
                command_id = %command.id,
                unit_id = %command.unit_id,
                "simulating restart"
            );
        }
        FleetCommandKind::DoWork => {
            info!(
                command_id = %command.id,
                unit_id = %command.unit_id,
                "performing assigned work"
            );
        }
    }
}
