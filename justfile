# Fleet Management Challenge

default:
    @just --list

# Run the server
server:
    cargo run --bin server

# Run a single agent
agent:
    cargo run --bin agent

# Run a named agent
agent-named name:
    FLEET_AGENT_NAME={{name}} cargo run --bin agent

# Run the server and an agent together
run:
    @just server &
    @sleep 2
    @just agent

# Run benchmarks (default: 100 agents, 30s)
bench agents="100" duration="30":
    BENCH_AGENTS={{agents}} BENCH_DURATION={{duration}} cargo run --release --bin bench

# Build all binaries
build:
    cargo build

# Build release binaries
build-release:
    cargo build --release

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Run all checks (format, lint, build)
check:
    just fmt-check
    just lint
    just build

# Build Docker image
docker-build:
    docker build -t fleet-management .

# Run with Docker Compose
docker-up:
    docker compose up --build

# Stop Docker Compose
docker-down:
    docker compose down

# Run multiple agents
agents count="3":
    #!/usr/bin/env bash
    for i in $(seq 1 {{count}}); do
        FLEET_AGENT_NAME="agent-$i" cargo run --bin agent &
    done
    wait
