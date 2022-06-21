#!/bin/bash

set -euxo pipefail

. deps/spl-token-swap/assignment.conf

CTRL_PID=JPCtrLreUqsEbdhtxZ8zpd8wBydKz4nuEjX5u9Eg5H8
MRGN_PID=JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ
POOL_PID=JPPooLEqRo3NCSx82EdE2VZY5vUaSsgskpZPBHNGVLZ
META_PID=JPMetawzxw7WyH3qHUVScYHWFBGhjwqDnM2R9qVbRLp
MGNSWAP_PID=JPMAa5dnWLFRvUsumawFcGhnwikqZziLLfqn9SLNXPN
SPLSWAP_PID=SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8
ORCAv1_PID=DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1
ORCAv2_PID=9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP

CTRL_SO=target/deploy/jet_control.so
MRGN_SO=target/deploy/jet_margin.so
POOL_SO=target/deploy/jet_margin_pool.so
META_SO=target/deploy/jet_metadata.so
MGNSWAP_SO=target/deploy/jet_margin_swap.so
SPLSWAP_SO=$SPL_V21_FROM_SOURCE # this isn't the right version but it's the best i could do
ORCAv1_SO=$ORCA_V1_MAINNET
ORCAv2_SO=$ORCA_V2_MAINNET

COMPILE_FEATURES='testing'

main() {
    anchor build --skip-lint -p jet_control     -- --features $COMPILE_FEATURES
    anchor build --skip-lint -p jet_margin      -- --features $COMPILE_FEATURES
    anchor build --skip-lint -p jet_metadata    -- --features $COMPILE_FEATURES
    anchor build --skip-lint -p jet_margin_pool -- --features $COMPILE_FEATURES
    anchor build --skip-lint -p jet_margin_swap -- --features $COMPILE_FEATURES

    solana-test-validator -r \
        --bpf-program $CTRL_PID $CTRL_SO \
        --bpf-program $MRGN_PID $MRGN_SO \
        --bpf-program $POOL_PID $POOL_SO \
        --bpf-program $META_PID $META_SO \
        --bpf-program $MGNSWAP_PID $MGNSWAP_SO \
        --bpf-program $SPLSWAP_PID $SPLSWAP_SO \
        --bpf-program $ORCAv1_PID $ORCAv1_SO \
        --bpf-program $ORCAv2_PID $ORCAv2_SO \
        > /dev/null &
    sleep 4

    RUST_BACKTRACE=1 cargo test \
        --features localnet \
        --package hosted-tests \
        --exact --nocapture
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
