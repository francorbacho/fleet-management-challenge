use std::net::SocketAddr;
use std::sync::Arc;

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
    };
    let listener = TcpListener::bind(addr).await?;

    info!(%addr, "fleet management API listening");
    axum::serve(listener, router(state)).await?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("fleet_management_challenge=info,tower_http=info"));

    fmt().with_env_filter(filter).init();
}
