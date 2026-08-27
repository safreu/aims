#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/common.sh"
source "$SCRIPT_DIR/auth.sh"
source "$SCRIPT_DIR/households.sh"
source "$SCRIPT_DIR/devices.sh"
source "$SCRIPT_DIR/inventory.sh"
source "$SCRIPT_DIR/qr.sh"
source "$SCRIPT_DIR/shopping.sh"
source "$SCRIPT_DIR/events.sh"

setup_smoke_test

echo
echo "Running aims API smoke tests"
echo "Base URL: $BASE_URL"
echo

smoke_auth
smoke_households
smoke_devices
smoke_inventory
smoke_shopping
smoke_events
smoke_qr

smoke_device_revocation

echo
echo "================================="
echo "All aims smoke tests passed."
echo "================================="
echo
