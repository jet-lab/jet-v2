#!/bin/bash

set -e

if [[ ${SOLANA_LOGS:-false} == true ]]; then
	solana -ul logs &
fi

cargo run --bin jetctl -- test init-env -ul --no-confirm localnet.toml
cargo run --bin jetctl -- test generate-app-config -ul --no-confirm localnet.toml -o app/public/localnet.config.json
cargo run --bin jet-oracle-mirror -- -s $SOLANA_MAINNET_RPC -tl &
sleep 5


# run the typescript tests
npx ts-mocha -p ./tests/tsconfig.json -t 1000000 --paths 'tests/**/*.test.ts'
