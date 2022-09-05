#!/bin/bash

set -euxo pipefail

PAYER=~/.config/solana/devnet-payer.json

EVENT_QUEUE_KEYPAIR=$PWD/tests/deps/keypairs/event_queue-keypair.json
BIDS_KEYPAIR=$PWD/tests/deps/keypairs/bids-keypair.json
ASKS_KEYPAIR=$PWD/tests/deps/keypairs/asks-keypair.json

CRANK_KEYPAIR=$PWD/tests/deps/keypairs/crank-keypair.json
TEST_MINT_KEYPAIR=$PWD/tests/deps/keypairs/test_mint-keypair.json
ALICE_KEYPAIR=$PWD/tests/deps/keypairs/alice-keypair.json
BOB_KEYPAIR=$PWD/tests/deps/keypairs/bob-keypair.json

main() {
    local test_mint=$(solana-keygen pubkey $TEST_MINT_KEYPAIR)
    local bond_manager_seed=7237125616417531254
    local bond_manager_key=$(cargo run -p bonds-deploy -- generate-pubkey bond-manager \
        --mint $test_mint \
        --seed $bond_manager_seed)

    cargo run -p bonds-test-framework-cli -- \
        -p $PAYER \
        -u devnet \
        create-test-user \
            --keypair $ALICE_KEYPAIR \
            --bond-manager $bond_manager_key \
            --test-mint-keypair $TEST_MINT_KEYPAIR

    cargo run -p bonds-test-framework-cli -- \
        -p $PAYER \
        -u devnet \
        create-test-user \
            --keypair $BOB_KEYPAIR \
            --bond-manager $bond_manager_key \
            --test-mint-keypair $TEST_MINT_KEYPAIR

    echo "Pre-test balance" &>> crank-benchmark.log
    echo $(solana -u devnet account $PAYER) &>> crank-benchmark.log
    cargo run -p bonds-test-framework-cli -- \
        -p $PAYER \
        -u devnet \
        crank-benchmark \
            --bond-manager-key $bond_manager_key \
            --event-queue-keypair $EVENT_QUEUE_KEYPAIR \
            --bids-keypair $BIDS_KEYPAIR \
            --asks-keypair $ASKS_KEYPAIR \
            --test-mint-keypair $TEST_MINT_KEYPAIR \
            --crank-signer $CRANK_KEYPAIR \
            --alice $ALICE_KEYPAIR \
            --bob $BOB_KEYPAIR &>> crank-benchmark.log
    echo "Post-test balance" &>> crank-benchmark.log
    echo $(solana -u devnet account $PAYER) &>> crank-benchmark.log
}

main