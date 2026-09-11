#!/usr/bin/env bash
set -euo pipefail

export DATABASE_URL="postgres://${DATABASE_USER}:${DATABASE_PASSWORD}@${DATABASE_HOST}:${DATABASE_PORT}/${DATABASE_NAME}"

exec cargo sqlx "$@"