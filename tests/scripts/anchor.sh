#!/bin/bash

set -e

if [[ ${SOLANA_LOGS:-false} == true ]]; then
	solana -ul logs &
fi

cargo run --bin jetctl -- apply -ul --no-confirm config/localnet/
cargo run --bin jetctl -- generate-app-config -ul --no-confirm config/localnet -o apps/react-app/public/localnet.config.json
cargo run --bin jet-oracle-mirror -- -s ${SOLANA_MAINNET_RPC:='https://solana-api.projectserum.com'} -tl &

echo "waiting for oracles ..."

	while true; do
		if [[ -f tests/oracle-mirror.pid ]]; then
			break;
		fi
		sleep 5
	done
	echo "oracles ready!"

yarn build --force

cp apps/react-app/public/localnet.config.json apps/react-app/build/localnet.config.json

cp apps/react-app/public/localnet.config.legacy.json apps/react-app/build/localnet.config.legacy.json

cd apps/react-app
npx cypress run --record
cd ../..
yarn --cwd apps/react-app e2e:ci
