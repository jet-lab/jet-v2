#!/bin/bash

set -euxo pipefail

. $(dirname ${BASH_SOURCE[0]})/localnet_lib.sh

SOLANA_LOGS=true

anchor-build
test-file swap
test-file liquidate
test-file pool_overpayment
test-file rounding
test-file sanity
test-file load
