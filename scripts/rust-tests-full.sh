#!/bin/bash

set -euxo pipefail

PAYER_KEYPAIR=~/.config/solana/id.json
CRANK_KEYPAIR=$PWD/tests/deps/keypairs/crank-keypair.json

JET_BONDS_SO=$PWD/target/deploy/jet_bonds.so
METADATA_SO=$PWD/target/deploy/bonds_metadata.so

main() {
    local jet_bonds_pid="JBond79m9K6HqYwngCjiJHb311GTXggo46kGcT2GijUc"
    local metadata_pid="C8GWmni61jTvtdon55LJ5zkVGzyJuv5Mkq41YVaeyhGQ"
    local crank_signer_key="4Nb92rP6BdRzATGwB4aSZYsYsy1Z5o7ZzcNGY1u4Rnwe"
    
    # Build the *.so files for the validator
    anchor build --skip-lint -p jet_bonds -- --features mock-margin
    anchor build --skip-lint -p jet_bonds_metadata -- --features devnet

    #
    # Start the local validator.
    #
    solana-test-validator -r \
        --bpf-program $jet_bonds_pid $JET_BONDS_SO \
        --bpf-program $metadata_pid $METADATA_SO \
        > test-validator.log &
    sleep 8
    solana -ul logs &

    RUST_BACKTRACE=1 cargo test -p bonds-test-framework -- tests::integrated::localhost_full --exact --nocapture
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
cargo fmt --all --check
cargo clippy --all-targets -- -Dwarnings
main
