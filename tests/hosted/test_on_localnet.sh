#!/bin/bash

set -euxo pipefail

. $(dirname ${BASH_SOURCE[0]})/localnet_lib.sh

SOLANA_LOGS=true

anchor-build
cargo-test
