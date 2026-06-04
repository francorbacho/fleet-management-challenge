use fleet_management_challenge::domain::{
    AgentId, ComputeAssignment, ComputeCalculation, ComputeSubmission, FleetCommand,
    FleetCommandKind, FleetUnit, JobId, NewFleetUnit, display_agent_id, display_job_id,
    format_job_id,
};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let base_url =
        std::env::var("FLEET_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_owned());
    let agent_name =
        std::env::var("FLEET_AGENT_NAME").unwrap_or_else(|_| "fleet-agent".to_owned());

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

struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    fn new(base_url: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_owned();
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    async fn register_with_retry(
        &self,
        agent_name: &str,
    ) -> Result<FleetUnit, Box<dyn std::error::Error>> {
        let url = format!("{}/fleet", self.base_url);
        let mut attempts = 0;

        loop {
            match self
                .client
                .post(&url)
                .json(&NewFleetUnit {
                    name: agent_name.to_owned(),
                })
                .send()
                .await
            {
                Ok(response) => {
                    let agent = response.error_for_status()?.json::<FleetUnit>().await?;
                    return Ok(agent);
                }
                Err(error) => {
                    attempts += 1;
                    warn!(%error, attempts, "failed to register, retrying");
                    if attempts >= 10 {
                        return Err(error.into());
                    }
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    async fn next_command(
        &self,
        agent_id: AgentId,
    ) -> Result<Option<FleetCommand>, reqwest::Error> {
        let url = format!("{}/fleet/{}/commands/next", self.base_url, agent_id);
        let response = self.client.get(&url).send().await?.error_for_status()?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        response.json::<FleetCommand>().await.map(Some)
    }

    async fn submit_result(
        &self,
        agent_id: AgentId,
        job_id: JobId,
        result: f64,
    ) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/fleet/{}/jobs/{}/submit",
            self.base_url,
            agent_id,
            format_job_id(job_id)
        );

        self.client
            .post(&url)
            .json(&ComputeSubmission { result })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

/// Returns `true` if the agent should restart (re-register).
async fn handle_command(api: &ApiClient, command: FleetCommand) -> bool {
    match command.kind {
        FleetCommandKind::Diagnostics => {
            let diagnostics = collect_diagnostics();
            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                %diagnostics,
                "diagnostics complete"
            );
            false
        }
        FleetCommandKind::Restart => {
            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                "restarting agent"
            );
            true
        }
        FleetCommandKind::Compute => {
            let Some(ref assignment) = command.compute else {
                warn!(
                    job_id = %display_job_id(command.job_id),
                    "compute command missing assignment"
                );
                return false;
            };
            let result = calculate(assignment);

            info!(
                job_id = %display_job_id(command.job_id),
                agent_id = %display_agent_id(command.agent_id),
                number = assignment.number,
                calculation = ?assignment.calculation,
                result,
                "completed assigned compute job"
            );

            sleep(Duration::from_secs(5)).await;

            if let Err(error) = api
                .submit_result(command.agent_id, command.job_id, result)
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

fn calculate(assignment: &ComputeAssignment) -> f64 {
    match assignment.calculation {
        ComputeCalculation::Double => assignment.number * 2.0,
        ComputeCalculation::Square => assignment.number * assignment.number,
        ComputeCalculation::SquareRoot => assignment.number.sqrt(),
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
