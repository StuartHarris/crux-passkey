#! /bin/bash

set -euo pipefail

# shellcheck source=.env
source .env

# see https://www.section.io/engineering-education/how-to-get-ssl-https-for-localhost/
# for how to generate certs that are issued by a local CA.
# you'll need to add the CA to your browser's trust store (or trust them in keychain on macos)
# (spin 2.0 crashes on use of self-signed certs)
key="localhost_key.pem"
cert="localhost_cert.pem"

export OPENSSL_STATIC=1
export OPENSSL_DIR
OPENSSL_DIR=$(pwd)/webauthn/openssl_wasm/precompiled/

(
  cd ../web-leptos/ &&
    trunk build --release &&
    cp dist/* ../crux-passkey-server/static/
)

export SPIN_VARIABLE_RP_ID="$SPIN_VARIABLE_DOMAIN_LOCAL"

spin build --up \
  --listen '0.0.0.0:443' \
  --tls-key $key \
  --tls-cert $cert \
  --sqlite @migration.sql
