#!/usr/bin/env bash

# Script that automates the initialization process of the DB backend. Overwrite
# the following variables for a deployment scenario. Default values are only
# advised for development scenarios.

set -eo pipefail

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 "
    cargo install --version=0.7.4 sqlx-cli --no-default-features --features mysql"
    echo >&2 "to install it."
    exit 1
fi

# Check if a custom user has been set, otherwise default to 'mariadb'
DB_USER="${MARIADB_USER:=user}"
# Check if a custom password has been set, otherwise default to 'password'
DB_PASSWORD="${MARIADB_PASSWORD:=password}"
# Check if a custom root password has been set, otherwise default to 'password'
DB_ROOT_PASSWORD="${MARIADB_ROOT_PASSWORD:=password}"
# Check if a custom database name has been set, otherwise default to 'test'
DB_NAME="${MARIADB_DB:=test_cocktail}"
# Check if a custom port has been set, otherwise default to '3306'
DB_PORT="${MARIADB_PORT:=3306}"
# Check if a custom host has been set, otherwise default to 'localhost'
DB_HOST="${MARIADB_HOST:=127.0.0.1}"

# Allow to skip Docker if the system already includes a MariaDB server.
if [[ -z "${SKIP_DOCKER}" ]]
then
    # Check if an instance of the MariaDB image is already running.
    while [ $(docker ps | grep mariadb -c) -le 0 ]
    do
        echo >&2 "Docker container ID:"
        docker run \
            -e MARIADB_USER=${DB_USER} \
            -e MARIADB_ROOT_PASSWORD=${DB_PASSWORD} \
            -e MARIADB_DATABASE=${DB_NAME} \
            -e MARIADB_ROOT_PASSWORD=${DB_ROOT_PASSWORD} \
            -p "${DB_PORT}":3306 \
            -d mariadb

        # Timeout to let the DB backend boot up
        sleep 10
    done

    echo >&2 "DB instance running"
fi

export DATABASE_URL=mariadb://$root:${DB_ROOT_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run
echo >&2 "Migrations executed"
echo >&2 "MariaDB ready!"
