#!/bin/bash
# Execute all validations in this repository

set -euxo pipefail

cargo fmt -- --check
cargo clippy --all-targets -- -Dwarnings
prettier --check libraries/ts
cargo test
tests/hosted/test_on_localnet.sh
anchor test --skip-lint -- --features testing

set +x
echo -e '\n\n ✔ all good'
