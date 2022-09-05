#!/bin/bash

set -euxo pipefail

PAYER=~/.config/solana/devnet-payer.json

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

JET_BONDS_SO=$PWD/target/deploy/jet_bonds.so
METADATA_SO=$PWD/target/deploy/bonds_metadata.so

main() {
    local spl_token_pid="TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

    anchor build

    echo "Pre-deployment" &>> deployment-metrics.txt 
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt &


    echo "Deploying Bonds" &>> deployment-metrics.txt
    solana deploy -u devnet $JET_BONDS_SO $JET_BONDS_KEYPAIR
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt &

    echo "Deploying Metadata" &>> deployment-metrics.txt
    solana deploy -k $PAYER -u devnet $METADATA_SO $BONDS_METADATA_KEYPAIR
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt &

    echo "Deploying Event Queue" &>> deployment-metrics.txt
    local eq_key=$(solana-keygen pubkey $EVENT_QUEUE_KEYPAIR)
    cargo run -p bonds-deploy -- deploy-keypair-account \
        -p $PAYER \
        -u devnet \
        event-queue --keypair-file $EVENT_QUEUE_KEYPAIR
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt

    echo "Deploying Bids Account" &>> deployment-metrics.txt
    local bids_key=$(solana-keygen pubkey $BIDS_KEYPAIR)
    cargo run -p bonds-deploy -- deploy-keypair-account \
        -p $PAYER \
        -u devnet \
        orderbook-slab --keypair-file $BIDS_KEYPAIR
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt

    echo "Deploying Asks Account" &>> deployment-metrics.txt
    local asks_key=$(solana-keygen pubkey $ASKS_KEYPAIR)
    cargo run -p bonds-deploy -- deploy-keypair-account \
        -p $PAYER \
        -u devnet \
        orderbook-slab --keypair-file $ASKS_KEYPAIR
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt

    echo "Deploying Test Mint" &>> deployment-metrics.txt
    local test_mint=$(solana-keygen pubkey $TEST_MINT_KEYPAIR)
    cargo run -p bonds-test-framework -- \
        -p $PAYER \
        -u devnet \
        create-test-mint \
            --keypair $TEST_MINT_KEYPAIR
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt


    echo "Deploying Bond Manager" &>> deployment-metrics.txt
    local bond_manager_seed=7237125616417531254
    cargo run -p bonds-deploy -- initialize-program-state \
        -p $PAYER \
        -u devnet \
        bond-manager \
            --authority $AUTHORITY_KEYPAIR \
            --mint $test_mint \
            --seed $bond_manager_seed \
            --version-tag 0 \
            --conversion-decimals 0 \
            --duration 3
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt

    local bond_manager_key=$(cargo run -p bonds-deploy -- generate-pubkey bond-manager \
        --mint $test_mint \
        --seed $bond_manager_seed)

    echo "Deploying Orderbook" &>> deployment-metrics.txt
    cargo run -p bonds-deploy -- initialize-program-state \
        -p $PAYER \
        -u devnet \
        orderbook \
            --authority $AUTHORITY_KEYPAIR \
            --bond-manager-key $bond_manager_key \
            --event-queue-key $eq_key \
            --bids-key $bids_key \
            --asks-key $asks_key \
            --minimum-order-size 100
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt

    echo "Deploying Crank Metadata" &>> deployment-metrics.txt
    cargo run -p bonds-deploy -- initialize-program-state \
        -p $PAYER \
        -u devnet \
        crank-metadata \
            --authority $AUTHORITY_KEYPAIR \
            --crank-signer $CRANK_KEYPAIR
    echo $(solana -u devnet account $PAYER) &>> deployment-metrics.txt
}

main