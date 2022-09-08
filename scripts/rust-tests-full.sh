#!/bin/bash

set -e

cargo fmt --all --check
cargo clippy --all-targets -- -Dwarnings
cargo test