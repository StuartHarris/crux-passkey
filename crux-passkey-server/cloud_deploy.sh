#! /bin/bash

set -euo pipefail

# shellcheck source=.env
source .env

export OPENSSL_STATIC=1
export OPENSSL_DIR
OPENSSL_DIR=$(pwd)/webauthn/openssl_wasm/precompiled/

spin cloud deploy --build --variable rp_id="$SPIN_VARIABLE_DOMAIN_REMOTE"
