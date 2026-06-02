use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

use fleet_management_challenge::api::{AppState, router};
use fleet_management_challenge::registry::InMemoryFleetRegistry;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let addr = std::env::var("FLEET_API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
        .parse::<SocketAddr>()?;

    let state = AppState {
        registry: Arc::new(InMemoryFleetRegistry::default()),
        command_queues: Arc::new(Mutex::default()),
        jobs: Arc::new(Mutex::default()),
    };
    let listener = TcpListener::bind(addr).await?;

    info!(%addr, "fleet management API listening");
    axum::serve(listener, router(state)).await?;

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
