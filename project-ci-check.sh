#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cargo test --manifest-path "$project_root/Cargo.toml" --locked
cargo test --manifest-path "$project_root/Cargo.toml" --locked --all-features
cargo run --manifest-path "$project_root/tests/fixtures/inventory_consumer/Cargo.toml" --locked
