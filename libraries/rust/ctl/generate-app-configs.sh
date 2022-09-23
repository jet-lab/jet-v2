#!/bin/bash

CFG_DEVNET=./generated/devnet.json
CFG_MAINNET=./generated/mainnet.json
CFG_GLOBAL=./generated/config.json.gen

if [[ $(type -P "jetctl") ]]; then
    JETCTL=jetctl
elif [[ -f ./target/debug/jetctl ]]; then
    JETCTL=./target/debug/jetctl
else
    JETCTL=./target/release/jetctl
fi

mkdir -p ./generated
$JETCTL generate-app-config ./configs/devnet -ud -o $CFG_DEVNET
$JETCTL generate-app-config ./configs/mainnet -um -o $CFG_MAINNET

echo '{}' > $CFG_GLOBAL
echo $(jq '.devnet = input' $CFG_GLOBAL $CFG_DEVNET) > $CFG_GLOBAL
echo $(jq '."mainnet-beta" = input' $CFG_GLOBAL $CFG_MAINNET) > $CFG_GLOBAL

jq '.' $CFG_GLOBAL > ./generated/config.json
rm -f $CFG_GLOBAL