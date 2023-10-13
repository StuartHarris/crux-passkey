#! /bin/bash

# shellcheck source=.env
source .env

spin cloud deploy --variable api_key="$SPIN_CONFIG_API_KEY"

# note you'll need to substutite "generous-lion" with the name of your database instance
spin cloud sqlite execute generous-lion "$(cat migration.sql)"
