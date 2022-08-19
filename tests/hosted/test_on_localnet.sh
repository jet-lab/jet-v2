#!/bin/bash

set -euxo pipefail

. $(dirname ${BASH_SOURCE[0]})/localnet_lib.sh

anchor-build
test combined
