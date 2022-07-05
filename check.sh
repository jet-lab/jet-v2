#!/bin/bash
# Execute all validations in this repository

set -euxo pipefail

cargo fmt -- --check
cargo clippy --all-targets -- -Dwarnings
prettier --check .
eslint . --ext ts
cargo test
tests/hosted/test_on_localnet.sh
anchor test --skip-lint -- --features testing
