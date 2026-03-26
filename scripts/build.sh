#!/bin/bash
set -e

echo "Building FlashPool v2 contracts..."

cd "$(dirname "$0")/../contracts"

echo "Building treasury..."
cargo build --release -p flash-treasury --target wasm32-unknown-unknown

echo "Building campaign manager v2..."
cargo build --release -p flash-campaign-manager-v2 --target wasm32-unknown-unknown

echo ""
echo "Build complete. WASM artifacts:"
ls -lh target/wasm32-unknown-unknown/release/flash_treasury.wasm 2>/dev/null || echo "  flash_treasury.wasm - check target directory"
ls -lh target/wasm32-unknown-unknown/release/flash_campaign_manager_v2.wasm 2>/dev/null || echo "  flash_campaign_manager_v2.wasm - check target directory"
