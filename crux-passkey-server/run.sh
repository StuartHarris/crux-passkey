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

spin build --up \
  --listen '0.0.0.0:3000' \
  --tls-key $key \
  --tls-cert $cert \
  --sqlite @migration.sql
