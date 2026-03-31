#!/usr/bin/env bash
# setup.sh — Verify toolchain presence and run bootstrap commands.
set -euo pipefail

echo "=== CortexOS Development Setup ==="
echo ""

# Check Rust
if command -v cargo &>/dev/null; then
    RUST_VER="$(rustc --version 2>/dev/null || echo 'unknown')"
    echo "[OK] Rust: $RUST_VER"
else
    echo "[MISSING] Rust toolchain not found"
    echo "  Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.85.0"
    exit 1
fi

# Check Node.js
if command -v node &>/dev/null; then
    NODE_VER="$(node --version 2>/dev/null)"
    echo "[OK] Node.js: $NODE_VER"
else
    echo "[MISSING] Node.js not found"
    echo "  Install: https://nodejs.org/ (v22 LTS recommended)"
    exit 1
fi

# Check pnpm
if command -v pnpm &>/dev/null; then
    PNPM_VER="$(pnpm --version 2>/dev/null)"
    echo "[OK] pnpm: $PNPM_VER"
else
    echo "[MISSING] pnpm not found"
    echo "  Install: corepack enable && corepack prepare pnpm@9.15.0 --activate"
    exit 1
fi

echo ""
echo "=== Installing Rust dependencies ==="
cargo build --workspace 2>&1 || {
    echo "[FAIL] cargo build failed"
    exit 1
}
echo "[OK] Rust workspace builds"

echo ""
echo "=== Installing frontend dependencies ==="
pnpm install 2>&1 || {
    echo "[FAIL] pnpm install failed"
    exit 1
}
echo "[OK] Frontend dependencies installed"

echo ""
echo "=== Setup complete ==="
