#! /bin/bash

# shellcheck source=.env
source .env

spin deploy --variable api_key="$SPIN_CONFIG_API_KEY" --sqlite @migration.sql
