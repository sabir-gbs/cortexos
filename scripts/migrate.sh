#!/usr/bin/env bash
# migrate.sh — Invoke the database migration path.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MIGRATION_DIR="$REPO_ROOT/crates/cortex-db/migrations"

echo "=== CortexOS Database Migration ==="

if [ ! -d "$MIGRATION_DIR" ]; then
    echo "ERROR: Migration directory not found at $MIGRATION_DIR"
    exit 1
fi

# List available migrations
echo "Available migrations:"
for f in "$MIGRATION_DIR"/*.up.sql; do
    [ -f "$f" ] || continue
    basename "$f"
done

echo ""
echo "NOTE: Migration execution requires a running database."
echo "      For now, migrations are validated at compile time via cortex-db."
echo "      Run 'cargo build -p cortex-db' to validate migration SQL syntax."
cargo build -p cortex-db 2>&1 || {
    echo "[FAIL] cortex-db build failed — check migration SQL"
    exit 1
}

echo "[OK] Migrations validated"
