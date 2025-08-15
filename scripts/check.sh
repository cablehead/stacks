#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")/../src-tauri"

echo "🎨 Checking formatting..."
cargo fmt --check

echo "📎 Running clippy..."
cargo clippy -- -D warnings

echo "🧪 Running tests..."
cargo test

echo "✅ All checks passed!"