#!/bin/bash

set -e

if [[ ${SOLANA_LOGS:-false} == true ]]; then
	solana -ul logs &
fi

cargo run --bin jetctl -- test init-env -ul --no-confirm localnet.toml
cargo run --bin jetctl -- test generate-app-config -ul --no-confirm localnet.toml -o app/public/localnet.config.json
cargo run --bin jet-oracle-mirror -- -s ${SOLANA_MAINNET_RPC:-'https://solana-api.projectserum.com'} -tl &

echo "waiting for oracles ..."

	while true; do
		if [[ -f tests/oracle-mirror.pid ]]; then
			break;
		fi
		sleep 5
	done
	echo "oracles ready!"

if [[ ${E2E:-false} == false ]]; then
	set +x
	whoami
	id -u
	ls -al /home/tools/.config/solana/
	npx ts-mocha -p ./tests/tsconfig.json -t 1000000 --paths 'tests/**/*.test.ts'
else
	cp app/public/localnet.config.json app/build/localnet.config.json
	yarn --cwd app e2e:ci
fi
