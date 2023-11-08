#! /bin/bash

# shellcheck source=.env
source .env

export OPENSSL_STATIC=1
export OPENSSL_DIR
OPENSSL_DIR=$(pwd)/webauthn/openssl_wasm/precompiled/

spin cloud sqlite execute @migration.sql --label default --app crux-passkey-server

spin cloud deploy --build
