# Fleet Management Challenge

A distributed fleet management system with a central server and agents that register, receive commands, and execute compute jobs.

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

## Docker

```sh
docker compose up --build
```

This starts 1 server and 3 agents. The dashboard is available at http://localhost:3000.


## Architecture

- **Server** — HTTP API (Axum) that manages agent registration, command queuing, and job tracking. Includes a web dashboard at `/`.
- **Agent** — Connects to the server, polls for commands, executes them, and submits results. Auto-reconnects on failure.
- **Bench** — Ping benchmark that spawns 50 agents, each performing 3 roundtrip pings to measure command-delivery latency.

### Conceptual Overview

The conceptual flow is simple.

1. Server starts
2. Agent register and connects
3. Agent starts polling
4. User requests an action to the server (`job=pending`)
5. Agent picks up task from the server through polling (`job=accepted`)
6. Agent starts and completes task
7. Agent submits task to the server through REST API (`job=succeed`)
8. User requests job state
9. Server replies with job result

# Steps to make it production ready

This implementation keeps the moving parts intentionally small: an in-memory registry,
per-agent command queues, and a simple job table. To make the same design production
ready, the next steps would be focused on durability, reliability, and device-specific
behavior.

- Replace the in-memory `FleetRegistry` implementation with persistent storage so agents,
	jobs, and results survive server restarts.
- Add device-specific commands and structured result payloads instead of the current demo
	command set.
- Add stronger handling for weak networks: reconnect backoff, command retries, idempotent
	submissions, and tests that simulate dropped or delayed connections.

In a larger fleet system, the agents would represent vehicles, drones, robots, or edge
devices. The server acts as the control plane: it tracks which units are available, queues
work for each unit, records job state transitions, and stores the final result reported by
the agent. Commands such as `Diagnostics`, `Ping`, `Restart`, and `Exit` map to common
operator needs: inspect a unit, measure command latency, recover a device, or remove it
from service.


## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `FLEET_API_ADDR` | `127.0.0.1:3000` | Server listen address |
| `FLEET_API_URL` | `http://127.0.0.1:3000` | Server URL (used by agents and bench) |
| `FLEET_AGENT_NAME` | `fleet-agent` | Agent display name |

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

