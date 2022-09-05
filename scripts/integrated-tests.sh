#!/bin/bash

set -euxo pipefail


# Program binaries
JET_BONDS_SO=$PWD/target/deploy/jet_bonds.so
METADATA_SO=$PWD/target/deploy/bonds_metadata.so

main() {
    local jet_bonds_pid="JBond79m9K6HqYwngCjiJHb311GTXggo46kGcT2GijUc"
    local metadata_pid="C8GWmni61jTvtdon55LJ5zkVGzyJuv5Mkq41YVaeyhGQ"

    # Build the *.so files for the validator
    anchor build --skip-lint -p jet_bonds -- --features mock-margin
    anchor build --skip-lint -p jet_bonds_metadata

    solana-test-validator -r \
        --bpf-program $jet_bonds_pid $JET_BONDS_SO \
        --bpf-program $metadata_pid $METADATA_SO \
        > test-validator.log &
    sleep 8
    solana -ul logs &

    # set up a state to run integration tests against
    # also starts the orderbook crank
    scripts/deploy-test-orderbook.sh

    # Build the wasm-utils
    wasm-pack build --target nodejs libraries/ts/wasm-utils
    npm i libraries/ts/wasm-utils/pkg

    npx ts-mocha -p ./tsconfig.json -t 1000000 tests/typescript/*.test.ts
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
