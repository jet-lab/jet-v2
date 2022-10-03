#!/bin/bash

CFG_DEVNET=./generated/devnet.json
CFG_MAINNET=./generated/mainnet.json
CFG_GLOBAL=./generated/config.json.gen

mkdir -p ./generated
cargo run --bin jetctl generate-app-config ./configs/devnet -ud -o $CFG_DEVNET
cargo run --bin jetctl generate-app-config ./configs/mainnet -um -o $CFG_MAINNET

echo '{}' > $CFG_GLOBAL
echo $(jq '.devnet = input' $CFG_GLOBAL $CFG_DEVNET) > $CFG_GLOBAL
echo $(jq '."mainnet-beta" = input' $CFG_GLOBAL $CFG_MAINNET) > $CFG_GLOBAL

jq '.' $CFG_GLOBAL > ./generated/config.json
rm -f $CFG_GLOBAL