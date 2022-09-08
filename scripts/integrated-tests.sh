#!/bin/bash

set -euxo pipefail

NETWORK="localhost"

# Program binaries
JET_BONDS_SO=$PWD/target/deploy/jet_bonds.so
JET_MARGIN_SO=$PWD/tests/deps/jet_margin.so
METADATA_SO=$PWD/target/deploy/bonds_metadata.so

TEST_MINT=$PWD/tests/deps/keypairs/test_mint-keypair.json

main() {
    local jet_bonds_pid="JBond79m9K6HqYwngCjiJHb311GTXggo46kGcT2GijUc"
    local margin_pid="JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ"
    local metadata_pid="C8GWmni61jTvtdon55LJ5zkVGzyJuv5Mkq41YVaeyhGQ"

    # Build the *.so files for the validator
    anchor build --skip-lint -p jet_bonds -- --features mock-margin
    anchor build --skip-lint -p jet_bonds_metadata

    solana-test-validator -r \
        --bpf-program $jet_bonds_pid $JET_BONDS_SO \
        --bpf-program $margin_pid $JET_MARGIN_SO \
        --bpf-program $metadata_pid $METADATA_SO \
        > test-validator.log &
    wait_for_validator
    solana -ul logs &

    # initialize a test mint
    spl-token -u localhost create-token \
         --decimals 6 \
         --mint-authority $TEST_MINT \
         $TEST_MINT
    local test_mint_key="$(solana-keygen pubkey $TEST_MINT)"

    # launch a new bond market
    cargo run -p jet-bonds-cli --bin main -- \
        deploy-market \
            --version 0 \
            --seed 0 \
            --duration 5 \
            --conversion-factor 0 \
            --min-base-order-size 1000 \
            --underlying-mint $test_mint_key

    # Build the wasm-utils
    wasm-pack build --target nodejs libraries/ts/wasm-utils
    npm i libraries/ts/wasm-utils/pkg

    npx ts-mocha -p ./tsconfig.json -t 1000000 tests/typescript/*.test.ts
}

isready() {
    solana ping -u localhost -c 1 &> /dev/null
}

wait_for_validator() {
    set +e
    for i in {0..10}; do
        isready
        if [ $? -eq 0 ]; then
            break
        else
            sleep 3
    fi
    done
    set -e
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
