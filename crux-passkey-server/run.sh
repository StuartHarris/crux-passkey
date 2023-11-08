#! /bin/bash

# shellcheck source=.env
source .env

key="key.pem"
cert="cert.pem"

if [ ! -e $key ] || [ ! -e $cert ]; then
  openssl req \
    -x509 \
    -newkey rsa:4096 \
    -keyout $key \
    -out $cert \
    -sha256 \
    -days 365 \
    -nodes \
    -subj "/CN=0.0.0.0"
fi

export OPENSSL_STATIC=1
export OPENSSL_DIR
OPENSSL_DIR=$(pwd)/webauthn/openssl_wasm/precompiled/

spin build --up \
  --listen '0.0.0.0:443' \
  --tls-key $key \
  --tls-cert $cert \
  --sqlite @migration.sql
