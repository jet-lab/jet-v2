#!/bin/bash

set -e

if [[ ${SOLANA_LOGS:-false} == true ]]; then
	solana -ul logs &
fi

# initialize some state on chain to test against
RUST_BACKTRACE=1 cargo run --package hosted-tests --bin launch_bonds

# run the typescript tests
npx ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.test.ts
