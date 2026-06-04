# Fleet Management Challenge

A distributed fleet management system with a central server and agents that register, receive commands, and execute compute jobs.

## Architecture

- **Server** — HTTP API (Axum) that manages agent registration, command queuing, and job tracking. Includes a web dashboard at `/`.
- **Agent** — Connects to the server, polls for commands, executes them, and submits results. Auto-reconnects on failure.
- **Bench** — Load testing tool that spawns many agents to measure server throughput.

## Quick Start

```sh
# Start the server
cargo run --bin server

# In another terminal, start an agent
cargo run --bin agent

# Alternatively, you can run a benchmark
cargo run --bin bench
```

Then open http://127.0.0.1:3000 in your browser.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `FLEET_API_ADDR` | `127.0.0.1:3000` | Server listen address |
| `FLEET_API_URL` | `http://127.0.0.1:3000` | Server URL (used by agents) |
| `FLEET_AGENT_NAME` | `fleet-agent` | Agent display name |
| `BENCH_AGENTS` | `100` | Number of benchmark agents |
| `BENCH_DURATION` | `30` | Benchmark duration in seconds |

## API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/` | Web dashboard |
| `GET` | `/health` | Health check |
| `GET` | `/fleet` | List all agents |
| `POST` | `/fleet` | Register a new agent |
| `GET` | `/fleet/{agent_id}` | Get agent details |
| `PUT` | `/fleet/{agent_id}/heartbeat` | Agent heartbeat |
| `POST` | `/fleet/{agent_id}/commands` | Queue a command |
| `GET` | `/fleet/{agent_id}/commands/next` | Long-poll for next command |
| `POST` | `/fleet/{agent_id}/jobs/{job_id}/submit` | Submit compute result |

## Commands

- **Double**: Agent performs a calculation (doubles a number) and submits the result
- **Ping**: Grabs a timestamp, sends a ping, and calculates the time it takes to come back
- **Diagnostics**: Agent reports system info (PID, OS, architecture, CPU count)
- **Restart**: Agent re-registers with the server under a new ID


## IDs

Agent and job IDs are hex-encoded 48-bit random values (e.g., `a#456362dbc8b3`, `j#25f9e2e48afe`).

## Docker

```sh
docker compose up --build
```

This starts 1 server and 3 agents. The dashboard is available at http://localhost:3000.
