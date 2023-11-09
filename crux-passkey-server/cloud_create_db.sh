#! /bin/bash

set -euo pipefail

spin cloud sqlite create auth-db
spin cloud sqlite execute @migration.sql --database auth-db
