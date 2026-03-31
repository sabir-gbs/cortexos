#!/usr/bin/env bash
# dev.sh — Start backend and frontend entrypoints.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Starting CortexOS development environment ==="

# Start backend (cargo run for the API crate)
echo "Starting backend..."
cd "$REPO_ROOT"
cargo run --bin cortex-api &
BACKEND_PID=$!

# Give backend a moment to start
sleep 2

# Start frontend dev server
echo "Starting frontend..."
pnpm dev &
FRONTEND_PID=$!

cleanup() {
    echo ""
    echo "Shutting down..."
    kill "$BACKEND_PID" 2>/dev/null || true
    kill "$FRONTEND_PID" 2>/dev/null || true
    exit 0
}
trap cleanup INT TERM

echo ""
echo "Backend PID: $BACKEND_PID"
echo "Frontend PID: $FRONTEND_PID"
echo "Press Ctrl+C to stop"

wait
