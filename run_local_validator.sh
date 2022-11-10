#!/bin/bash
export SOLANA_MAINNET_RPC=${SOLANA_MAINNET_RPC:-'https://solana-api.projectserum.com'}

case $1 in
    -r|--reset)
        exec ./tests/scripts/on_localnet.sh start-new-validator
    ;;
esac

exec ./tests/scripts/on_localnet.sh resume-validator
