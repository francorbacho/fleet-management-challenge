use fleet_management_challenge::domain::{
    CommandRequest, FleetCommand, FleetUnit, JobSubmission, NewFleetUnit, format_job_id,
};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

const AGENT_COUNT: usize = 50;
const PINGS_PER_AGENT: usize = 3;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let api_url =
        std::env::var("FLEET_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_owned());
    let base = api_url.trim_end_matches('/').to_owned();
    let client = reqwest::Client::new();

    // Check server is up.
    match client.get(format!("{}/health", base)).send().await {
        Ok(_) => info!("server is reachable"),
        Err(e) => {
            error!(%e, "server is not reachable at {}", base);
            return Err(e.into());
        }
    }

    info!(
        agents = AGENT_COUNT,
        pings = PINGS_PER_AGENT,
        "starting ping benchmark"
    );

    let mut handles = Vec::with_capacity(AGENT_COUNT);

    for i in 0..AGENT_COUNT {
        let client = client.clone();
        let base = base.clone();

        handles.push(tokio::spawn(async move {
            run_agent(&client, &base, i).await;
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    info!("benchmark complete");
    Ok(())
}

async fn run_agent(client: &reqwest::Client, base: &str, index: usize) {
    let name = format!("bench-agent-{}", index);

    // Register.
    let agent: FleetUnit = match client
        .post(format!("{}/fleet", base))
        .json(&NewFleetUnit { name: name.clone() })
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(r) => match r.json().await {
            Ok(a) => a,
            Err(e) => {
                error!(%e, agent = %name, "register parse failed");
                return;
            }
        },
        Err(e) => {
            error!(%e, agent = %name, "register failed");
            return;
        }
    };

    for ping_num in 0..PINGS_PER_AGENT {
        // Queue a ping.
        let cmd: FleetCommand = match client
            .post(format!("{}/fleet/{}/commands", base, agent.id))
            .json(&CommandRequest::Ping)
            .send()
            .await
            .and_then(|r| r.error_for_status())
        {
            Ok(r) => match r.json().await {
                Ok(c) => c,
                Err(e) => {
                    error!(%e, agent = %name, ping_num, "queue parse failed");
                    continue;
                }
            },
            Err(e) => {
                error!(%e, agent = %name, ping_num, "queue failed");
                continue;
            }
        };

        // Poll for the command (act as agent).
        let _received: FleetCommand = match client
            .get(format!("{}/fleet/{}/commands/next", base, agent.id))
            .send()
            .await
            .and_then(|r| r.error_for_status())
        {
            Ok(r) => match r.json().await {
                Ok(c) => c,
                Err(e) => {
                    error!(%e, agent = %name, ping_num, "poll parse failed");
                    continue;
                }
            },
            Err(e) => {
                error!(%e, agent = %name, ping_num, "poll failed");
                continue;
            }
        };

        // Submit result — server computes the latency.
        match client
            .post(format!(
                "{}/fleet/{}/jobs/{}/submit",
                base,
                agent.id,
                format_job_id(cmd.job_id)
            ))
            .json(&JobSubmission {
                result: "pong".to_owned(),
            })
            .send()
            .await
            .and_then(|r| r.error_for_status())
        {
            Ok(_) => info!(agent = %name, ping_num, "ping submitted"),
            Err(e) => error!(%e, agent = %name, ping_num, "submit failed"),
        }
    }

    // After pings, send an Exit command and submit an "exiting" result.
    let cmd: FleetCommand = match client
        .post(format!("{}/fleet/{}/commands", base, agent.id))
        .json(&CommandRequest::Exit)
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(r) => match r.json().await {
            Ok(c) => c,
            Err(e) => { error!(%e, agent = %name, "queue exit parse failed"); return; }
        },
        Err(e) => { error!(%e, agent = %name, "queue exit failed"); return; }
    };

    // Poll for the exit command.
    let _received: FleetCommand = match client
        .get(format!("{}/fleet/{}/commands/next", base, agent.id))
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(r) => match r.json().await {
            Ok(c) => c,
            Err(e) => { error!(%e, agent = %name, "exit poll parse failed"); return; }
        },
        Err(e) => { error!(%e, agent = %name, "exit poll failed"); return; }
    };

    match client
        .post(format!(
            "{}/fleet/{}/jobs/{}/submit",
            base,
            agent.id,
            format_job_id(cmd.job_id)
        ))
        .json(&JobSubmission { result: "exiting".to_owned() })
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(_) => info!(agent = %name, "exit submitted"),
        Err(e) => error!(%e, agent = %name, "exit submit failed"),
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
