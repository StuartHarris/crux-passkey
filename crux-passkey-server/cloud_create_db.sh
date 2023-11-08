#! /bin/bash

spin cloud sqlite create auth-db
spin cloud sqlite execute @migration.sql --database auth-db
