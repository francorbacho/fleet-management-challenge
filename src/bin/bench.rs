use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use fleet_management_challenge::domain::{FleetUnit, NewFleetUnit};
use tokio::time::{Duration, sleep};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let api_url =
        std::env::var("FLEET_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_owned());
    let agent_count: usize = std::env::var("BENCH_AGENTS")
        .unwrap_or_else(|_| "100".to_owned())
        .parse()?;
    let duration_secs: u64 = std::env::var("BENCH_DURATION")
        .unwrap_or_else(|_| "30".to_owned())
        .parse()?;

    info!(agents = agent_count, duration_secs, "starting benchmark");

    let registered = Arc::new(AtomicU64::new(0));
    let poll_count = Arc::new(AtomicU64::new(0));
    let poll_errors = Arc::new(AtomicU64::new(0));

    let start = Instant::now();
    let client = reqwest::Client::new();
    let mut handles = Vec::with_capacity(agent_count);

    for i in 0..agent_count {
        let api_url = api_url.clone();
        let client = client.clone();
        let registered = registered.clone();
        let poll_count = poll_count.clone();
        let poll_errors = poll_errors.clone();

        handles.push(tokio::spawn(async move {
            let agent_name = format!("bench-agent-{}", i);
            let registration_url = format!("{}/fleet", api_url.trim_end_matches('/'));

            let agent: FleetUnit = match client
                .post(&registration_url)
                .json(&NewFleetUnit { name: agent_name })
                .send()
                .await
            {
                Ok(resp) => match resp.error_for_status() {
                    Ok(resp) => match resp.json().await {
                        Ok(a) => a,
                        Err(_) => return,
                    },
                    Err(_) => return,
                },
                Err(_) => return,
            };

            registered.fetch_add(1, Ordering::Relaxed);

            let command_url = format!(
                "{}/fleet/{}/commands/next",
                api_url.trim_end_matches('/'),
                agent.id
            );

            let deadline = Instant::now() + Duration::from_secs(duration_secs);
            while Instant::now() < deadline {
                match client.get(&command_url).send().await {
                    Ok(_) => {
                        poll_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        poll_errors.fetch_add(1, Ordering::Relaxed);
                        sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start.elapsed();
    let total_registered = registered.load(Ordering::Relaxed);
    let total_polls = poll_count.load(Ordering::Relaxed);
    let total_errors = poll_errors.load(Ordering::Relaxed);

    info!(
        agents_registered = total_registered,
        total_polls,
        total_errors,
        elapsed_secs = elapsed.as_secs_f64(),
        polls_per_sec = total_polls as f64 / elapsed.as_secs_f64(),
        "benchmark complete"
    );

    Ok(())
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "info".to_owned());
    let filter = EnvFilter::new(filter);

    fmt().with_env_filter(filter).with_target(false).init();
}
