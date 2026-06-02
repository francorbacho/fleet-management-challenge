use fleet_management_challenge::domain::{FleetUnit, NewFleetUnit};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let api_url =
        std::env::var("FLEET_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_owned());
    let agent_name = std::env::var("FLEET_AGENT_NAME").unwrap_or_else(|_| "fleet-agent".to_owned());
    let registration_url = format!("{}/fleet", api_url.trim_end_matches('/'));

    let registered_unit = reqwest::Client::new()
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

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt().with_env_filter(filter).with_target(false).init();
}
