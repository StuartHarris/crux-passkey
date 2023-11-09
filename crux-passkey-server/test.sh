#! /bin/bash

set -euo pipefail

unset OPENSSL_STATIC
unset OPENSSL_DIR

cd webauthn
cargo nextest run
