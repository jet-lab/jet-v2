#!/bin/bash

set -e

if [[ ${SOLANA_LOGS:-false} == true ]]; then
	solana -ul logs &
fi

cargo run --bin jetctl -- apply -ul --no-confirm config/localnet/
solana-keygen new --no-bip39-passphrase -o apps/react-app/public/lookup-authority.json --force
solana airdrop -k apps/react-app/public/lookup-authority.json -ul 5
cargo run --bin jetctl -- generate-app-config -ul --no-confirm config/localnet/ -o apps/react-app/public/localnet.config.json --override-lookup-authority $(solana-keygen pubkey apps/react-app/public/lookup-authority.json)
cargo run --bin jet-alt-registry-client -- create-registry -ul --no-confirm --authority-path apps/react-app/public/lookup-authority.json -k apps/react-app/public/lookup-authority.json
cargo run --bin jet-alt-registry-client -- update-registry -ul --no-confirm --authority-path apps/react-app/public/lookup-authority.json -k apps/react-app/public/lookup-authority.json --airspace-name default
cargo run --bin jet-oracle-mirror -- -s ${SOLANA_MAINNET_RPC:='https://solana-api.projectserum.com'} -tl &

echo "waiting for oracles ..."

	while true; do
		if [[ -f tests/oracle-mirror.pid ]]; then
			break;
		fi
		sleep 5
	done
	echo "oracles ready!"

sleep 5
yarn build --force
cp apps/react-app/public/localnet.config.json apps/react-app/build/localnet.config.json
cp apps/react-app/public/localnet.config.legacy.json apps/react-app/build/localnet.config.legacy.json
sleep 5

yarn --cwd apps/react-app e2e:ci
