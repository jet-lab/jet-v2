#!/bin/bash

set -e

# initialize some state on chain to test against
cargo run --package hosted-tests --bin launch_bonds

# run the typescript tests
npx ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.test.ts
