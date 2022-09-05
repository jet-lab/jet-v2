#!/bin/bash

set -euxo pipefail

# Keypairs
PAYER=~/.config/solana/id.json

JET_BONDS_KEYPAIR=$PWD/target/deploy/jet_bonds-keypair.json
BONDS_METADATA_KEYPAIR=$PWD/target/deploy/bonds_metadata-keypair.json

# test related keypairs
AUTHORITY_KEYPAIR=$PWD/tests/deps/keypairs/authority-keypair.json
EVENT_QUEUE_KEYPAIR=$PWD/tests/deps/keypairs/event_queue-keypair.json
BIDS_KEYPAIR=$PWD/tests/deps/keypairs/bids-keypair.json
ASKS_KEYPAIR=$PWD/tests/deps/keypairs/asks-keypair.json
CRANK_KEYPAIR=$PWD/tests/deps/keypairs/crank-keypair.json
TEST_MINT_KEYPAIR=$PWD/tests/deps/keypairs/test_mint-keypair.json
ALICE_KEYPAIR=$PWD/tests/deps/keypairs/alice-keypair.json
BOB_KEYPAIR=$PWD/tests/deps/keypairs/bob-keypair.json


# Program binaries
JET_BONDS_SO=$PWD/target/deploy/jet_bonds.so
METADATA_SO=$PWD/target/deploy/bonds_metadata.so

main() {
    local spl_token_pid="TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

    solana-test-validator -r > test-validator.log &
    
    anchor build

    solana deploy -u localhost $JET_BONDS_SO $JET_BONDS_KEYPAIR
    solana deploy -u localhost $METADATA_SO $BONDS_METADATA_KEYPAIR

    local eq_key=$(solana-keygen pubkey $EVENT_QUEUE_KEYPAIR)
    cargo run -p bonds-deploy -- deploy-keypair-account \
        -p $PAYER \
        -u localhost \
        event-queue --keypair-file $EVENT_QUEUE_KEYPAIR
    local bids_key=$(solana-keygen pubkey $BIDS_KEYPAIR)
    cargo run -p bonds-deploy -- deploy-keypair-account \
        -p $PAYER \
        -u localhost \
        orderbook-slab --keypair-file $BIDS_KEYPAIR
    local asks_key=$(solana-keygen pubkey $ASKS_KEYPAIR)
    cargo run -p bonds-deploy -- deploy-keypair-account \
        -p $PAYER \
        -u localhost \
        orderbook-slab --keypair-file $ASKS_KEYPAIR
    
    local test_mint=$(solana-keygen pubkey $TEST_MINT_KEYPAIR)
    cargo run -p bonds-test-framework -- \
        -p $PAYER \
        -u localhost \
        create-test-mint \
            --keypair $TEST_MINT_KEYPAIR
    
    local bond_manager_seed=7237125616417531254
    cargo run -p bonds-deploy -- initialize-program-state \
        -p $PAYER \
        -u localhost \
        bond-manager \
            --authority $AUTHORITY_KEYPAIR \
            --mint $test_mint \
            --seed $bond_manager_seed \
            --version-tag 0 \
            --conversion-decimals 0 \
            --duration 3

    local bond_manager_key=$(cargo run -p bonds-deploy -- generate-pubkey bond-manager \
        --mint $test_mint \
        --seed $bond_manager_seed)
        
    cargo run -p bonds-deploy -- initialize-program-state \
        -p $PAYER \
        -u localhost \
        orderbook \
            --authority $AUTHORITY_KEYPAIR \
            --bond-manager-key $bond_manager_key \
            --event-queue-key $eq_key \
            --bids-key $bids_key \
            --asks-key $asks_key \
            --minimum-order-size 100

    cargo run -p bonds-deploy -- initialize-program-state \
        -p $PAYER \
        -u localhost \
        crank-metadata \
            --authority $AUTHORITY_KEYPAIR \
            --crank-signer $CRANK_KEYPAIR

    cargo run -p bonds-test-framework -- \
        -p $PAYER \
        -u localhost \
        create-test-user \
            --keypair $ALICE_KEYPAIR \
            --bond-manager $bond_manager_key \
            --test-mint-keypair $TEST_MINT_KEYPAIR

    cargo run -p bonds-test-framework -- \
        -p $PAYER \
        -u localhost \
        create-test-user \
            --keypair $BOB_KEYPAIR \
            --bond-manager $bond_manager_key \
            --test-mint-keypair $TEST_MINT_KEYPAIR

}

cleanup() {
    pkill -P $$ || true
    wait || true
}

trap_add() {
    trap_add_cmd=$1; shift || fatal "${FUNCNAME} usage error"
    for trap_add_name in "$@"; do
        trap -- "$(
            extract_trap_cmd() { printf '%s\n' "${3:-}"; }
            eval "extract_trap_cmd $(trap -p "${trap_add_name}")"
            printf '%s\n' "${trap_add_cmd}"
        )" "${trap_add_name}" \
            || fatal "unable to add to trap ${trap_add_name}"
    done
}

declare -f -t trap_add
trap_add 'cleanup' EXIT
main
