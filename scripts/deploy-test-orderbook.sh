#!/bin/bash

set -e

PAYER="$HOME/.config/solana/id.json"
EVENT_QUEUE_KEYPAIR=$PWD/tests/deps/keypairs/event_queue-keypair.json
AUTHORITY_KEYPAIR=$PWD/tests/deps/keypairs/authority-keypair.json
CRANK_AUTHORITY=$PWD/tests/deps/keypairs/crank-keypair.json
EVENT_QUEUE_KEYPAIR=$PWD/tests/deps/keypairs/event_queue-keypair.json
BIDS_KEYPAIR=$PWD/tests/deps/keypairs/bids-keypair.json
ASKS_KEYPAIR=$PWD/tests/deps/keypairs/asks-keypair.json
TEST_MINT_KEYPAIR=$PWD/tests/deps/keypairs/test_mint-keypair.json

deploy() {
    local eq_key=$(solana-keygen pubkey $EVENT_QUEUE_KEYPAIR)
    cargo run -p jet-bonds-cli -- deploy-keypair-account \
        -p $PAYER \
        -u localhost \
        event-queue --keypair-file $EVENT_QUEUE_KEYPAIR

    local bids_key=$(solana-keygen pubkey $BIDS_KEYPAIR)
    cargo run -p jet-bonds-cli -- deploy-keypair-account \
        -p $PAYER \
        -u localhost \
        orderbook-slab --keypair-file $BIDS_KEYPAIR
    local asks_key=$(solana-keygen pubkey $ASKS_KEYPAIR)
    cargo run -p jet-bonds-cli -- deploy-keypair-account \
        -p $PAYER \
        -u localhost \
        orderbook-slab --keypair-file $ASKS_KEYPAIR
    
    local test_mint=$(solana-keygen pubkey $TEST_MINT_KEYPAIR)
    cargo run -p jet-bonds-cli -- deploy-keypair-account \
        -p $PAYER \
        -u localhost \
        test-mint --keypair-file $TEST_MINT_KEYPAIR
    
    local bond_manager_seed=7237125616417531254
    cargo run -p jet-bonds-cli -- initialize-program-state \
        -p $PAYER \
        -u localhost \
        bond-manager \
            --authority $AUTHORITY_KEYPAIR \
            --mint $test_mint \
            --seed $bond_manager_seed \
            --version-tag 0 \
            --conversion-decimals 0 \
            --duration 3

    local bond_manager_key=$(cargo run -p jet-bonds-cli -- generate-pubkey bond-manager \
        --mint $test_mint \
        --seed $bond_manager_seed)        
    cargo run -p jet-bonds-cli -- initialize-program-state \
        -p $PAYER \
        -u localhost \
        orderbook \
            --authority $AUTHORITY_KEYPAIR \
            --bond-manager-key $bond_manager_key \
            --event-queue-key $eq_key \
            --bids-key $bids_key \
            --asks-key $asks_key \
            --minimum-order-size 100

    cargo run -p jet-bonds-cli -- initialize-program-state \
        -p $PAYER \
        -u localhost \
        crank-metadata \
            --authority $AUTHORITY_KEYPAIR \
            --crank-signer $CRANK_AUTHORITY

    cargo run -p jet-bonds-orderbook-crank -- \
        -u "http://localhost:8899" \
        -s $CRANK_AUTHORITY \
        -p $PAYER \
        --bond-manager-key $bond_manager_key \
    > crank.log &
}

deploy