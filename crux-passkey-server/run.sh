#! /bin/bash

# shellcheck source=.env
source .env

spin build --up --listen '0.0.0.0:3000' --sqlite @migration.sql
