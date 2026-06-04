use fleet_management_challenge::agent::ApiClient;
use fleet_management_challenge::domain::{
    CommandRequest, FleetCommand, FleetUnit, display_agent_id, display_job_id,
};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let base_url =
        std::env::var("FLEET_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_owned());
    let agent_name = std::env::var("FLEET_AGENT_NAME").unwrap_or_else(|_| "fleet-agent".to_owned());

    let api = ApiClient::new(base_url);

    loop {
        let agent = match api.register_with_retry(&agent_name).await {
            Ok(agent) => agent,
            Err(error) => {
                warn!(%error, "registration failed, retrying");
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        info!(
            agent_id = %display_agent_id(agent.id),
            agent_name = %agent.name,
            "agent registered"
        );

        run_session(&api, agent).await;

        info!("reconnecting to server");
        sleep(Duration::from_secs(2)).await;
    }
}

async fn run_session(api: &ApiClient, agent: FleetUnit) {
    let mut consecutive_errors = 0u32;

    loop {
        match api.next_command(agent.id).await {
            Ok(Some(command)) => {
                consecutive_errors = 0;
                let restart = handle_command(api, command).await;
                if restart {
                    info!("restart requested, re-registering");
                    return;
                }
            }
            Ok(None) => {
                consecutive_errors = 0;
            }
            Err(error) => {
                consecutive_errors += 1;
                warn!(%error, "command poll failed");
                if consecutive_errors >= 3 {
                    warn!("too many consecutive errors, re-registering");
                    return;
                }
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

/// Returns `true` if the agent should restart (re-register).
async fn handle_command(api: &ApiClient, command: FleetCommand) -> bool {
    match command.request {
        CommandRequest::Diagnostics => {
            let diagnostics = collect_diagnostics();
            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                %diagnostics,
                "diagnostics complete"
            );

            if let Err(error) = api
                .submit_result(command.agent_id, command.job_id, diagnostics)
                .await
            {
                warn!(
                    %error,
                    job_id = %display_job_id(command.job_id),
                    "failed to submit diagnostics result"
                );
            }

            false
        }
        CommandRequest::Restart => {
            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                "restarting agent"
            );
            true
        }
        CommandRequest::Double(number) => {
            let result = number * 2.0;

            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                number,
                result,
                "completed assigned compute job"
            );

            sleep(Duration::from_secs(5)).await;

            if let Err(error) = api
                .submit_result(command.agent_id, command.job_id, result.to_string())
                .await
            {
                warn!(
                    %error,
                    job_id = %display_job_id(command.job_id),
                    "failed to submit compute result"
                );
            }
            false
        }
    }
}

fn collect_diagnostics() -> String {
    let pid = std::process::id();
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    format!(
        "pid={} timestamp={} cpus={} os={} arch={}",
        pid,
        uptime,
        num_cpus,
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "info".to_owned());
    let filter = EnvFilter::new(filter);

    fmt().with_env_filter(filter).with_target(false).init();
}
