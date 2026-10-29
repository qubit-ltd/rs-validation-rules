#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
exec cargo run --locked --manifest-path "$project_root/tests/fixtures/inventory_consumer/Cargo.toml"
