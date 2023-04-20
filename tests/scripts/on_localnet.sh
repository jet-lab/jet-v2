#!/bin/bash

set -euxo pipefail

. deps/spl-token-swap/assignment.conf

CTRL_PID=JPCtrLreUqsEbdhtxZ8zpd8wBydKz4nuEjX5u9Eg5H8
MRGN_PID=JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ
POOL_PID=JPPooLEqRo3NCSx82EdE2VZY5vUaSsgskpZPBHNGVLZ
FIXED_TERM_PID=JBond79m9K6HqYwngCjiJHb311GTXggo46kGcT2GijUc
META_PID=JPMetawzxw7WyH3qHUVScYHWFBGhjwqDnM2R9qVbRLp
MRKT_PID=JBond79m9K6HqYwngCjiJHb311GTXggo46kGcT2GijUc
ASM_PID=JPASMkxARMmbeahk37H8PAAP1UzPNC4wGhvwLnBsfHi
JTS_PID=JPTSApMSqCHBww7vDhpaSmzipTV3qPg6vxub4qneKoy
MGNSWAP_PID=JPMAa5dnWLFRvUsumawFcGhnwikqZziLLfqn9SLNXPN
SPLSWAP_PID=SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8
ORCAv1_PID=DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1
ORCAv2_PID=9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP
SBR_PID=SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ
OPNBK_PID=EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj
LKPRG_PID=LTR8xXcSrEDsCbTWPY4JmJREFdMz4uYh65uajkVjzru

CTRL_SO=target/deploy/jet_control.so
MRGN_SO=target/deploy/jet_margin.so
POOL_SO=target/deploy/jet_margin_pool.so
META_SO=target/deploy/jet_metadata.so
MRKT_SO=target/deploy/jet_fixed_term.so
ASM_SO=target/deploy/jet_airspace.so
JTS_SO=target/deploy/jet_test_service.so
MGNSWAP_SO=target/deploy/jet_margin_swap.so
SPLSWAP_SO=$SPL_V20_FROM_CRATES
ORCAv1_SO=$ORCA_V1_MAINNET
ORCAv2_SO=$ORCA_V2_MAINNET
SBRSWAP_SO=deps/saber_stable_swap.so
OPNBK_SO=deps/openbook.so
LKPRG_SO=deps/lookup_table_registry.so

PROGRAM_FEATURES='testing'
TEST_FEATURES="${BATCH:-batch_all},localnet"
VALIDATOR_PID=

anchor-build() {
    anchor build $@ -- --features $PROGRAM_FEATURES
}

cargo-build() {
    RUST_BACKTRACE=1 cargo build \
        --features $TEST_FEATURES \
        --package hosted-tests
}

test() {
    RUST_BACKTRACE=1 with-validator cargo nextest run \
        --retries 2 \
        --features $TEST_FEATURES \
        --package hosted-tests \
        --test $@
}

run() {
    RUST_BACKTRACE=1 with-validator cargo run \
        --features $TEST_FEATURES \
        --package hosted-tests \
        --bin $@
}

init-idl() {
    cargo run --bin strip-idl-docs
    anchor idl init -f ./target/idl/jet_airspace.json $ASM_PID --provider.cluster localnet
    anchor idl init -f ./target/idl/jet_test_service.json $JTS_PID --provider.cluster localnet
    anchor idl init -f ./target/idl/jet_metadata.json $META_PID --provider.cluster localnet
    anchor idl init -f ./target/idl/jet_control.json $CTRL_PID --provider.cluster localnet
    # anchor idl init -f ./target/idl/jet_margin.json $MRGN_PID --provider.cluster localnet
    anchor idl init -f ./target/idl/jet_margin_pool.json $POOL_PID --provider.cluster localnet
    anchor idl init -f ./target/idl/jet_margin_swap.json $MGNSWAP_PID --provider.cluster localnet
    anchor idl init -f ./target/idl/jet_fixed_term.json $MRKT_PID --provider.cluster localnet
}

start-validator() {
    solana-test-validator \
        --bpf-program $JTS_PID $JTS_SO \
        --bpf-program $CTRL_PID $CTRL_SO \
        --bpf-program $MRGN_PID $MRGN_SO \
        --bpf-program $POOL_PID $POOL_SO \
        --bpf-program $META_PID $META_SO \
        --bpf-program $MRKT_PID $MRKT_SO \
        --bpf-program $ASM_PID $ASM_SO \
        --bpf-program $MGNSWAP_PID $MGNSWAP_SO \
        --bpf-program $SPLSWAP_PID $SPLSWAP_SO \
        --bpf-program $ORCAv1_PID $ORCAv1_SO \
        --bpf-program $ORCAv2_PID $ORCAv2_SO \
        --bpf-program $SBR_PID $SBRSWAP_SO \
        --bpf-program $OPNBK_PID $OPNBK_SO \
        --bpf-program $LKPRG_PID $LKPRG_SO \
        --quiet \
        $@ &
    VALIDATOR_PID=$!
    sleep ${VALIDATOR_STARTUP:-5}
}

start-oracle() {
    cargo run --bin jet-oracle-mirror -- -s $SOLANA_MAINNET_RPC -tl &
}

start-crank-service() {
    sleep 10
    mkdir -p .localnet

    log_filter="jet_margin_sdk=debug"
    env RUST_LOG=$log_filter cargo run --bin jet-fixed-terms-crank-service -- \
        --config-path apps/react-app/public/localnet.config.json \
        --log-path .localnet/crank.log \
        &
}

resume-validator() {
    start-validator
    start-crank-service
    start-oracle
    wait $VALIDATOR_PID
}

start-new-validator() {
    start-validator -r
    cargo run --bin jetctl -- apply -ul --no-confirm config/localnet/
    cargo run --bin jetctl -- generate-app-config -ul --no-confirm config/localnet/ -o apps/react-app/public/localnet.config.json
    start-crank-service
    start-oracle
    init-idl &
    wait $VALIDATOR_PID

}

with-validator() {
    start-validator -r
    if [[ ${SOLANA_LOGS:-false} == true ]]; then
        solana -ul logs &
    fi
    $@
}

with-resumed-validator() {
    start-validator
    if [[ ${SOLANA_LOGS:-false} == true ]]; then
        solana -ul logs &
    fi
    $@
}

kill_validator() {
    set +e
    [ -z $VALIDATOR_PID ] || (
        kill $VALIDATOR_PID
        pkill solana-test-validator
        killall solana-test-validator
    )
    return 0
}
trap kill_validator EXIT SIGHUP SIGINT SIGTERM


if (return 0 2>/dev/null); then
    echo sourced on_localnet.sh
else
    $@
fi
