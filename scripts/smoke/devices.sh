smoke_devices() {
    # -----------------------------------------------------------------------
    # Register device
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices" \
            -H "Content-Type: application/json" \
            -d '{
                "name": "Smoke Test Scanner",
                "kind": "scanner"
            }'
    )"

    assert_status "$STATUS" "201" "register device"

    DEVICE_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ -z "$DEVICE_ID" || "$DEVICE_ID" == "null" ]]; then
        echo "FAIL: device registration did not return an id"
        exit 1
    fi

    echo "PASS: device registration returned an id"

    # -----------------------------------------------------------------------
    # List devices
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices"
    )"

    assert_status "$STATUS" "200" "list devices"

    if ! jq -e \
        --arg id "$DEVICE_ID" \
        '.[] |
            select(
                .id == $id
                and .name == "Smoke Test Scanner"
                and .kind == "scanner"
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: registered device does not appear correctly in device list"
        exit 1
    fi

    echo "PASS: registered device appears in device list"

    # -----------------------------------------------------------------------
    # Rename device
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices/$DEVICE_ID" \
            -H "Content-Type: application/json" \
            -d '{
                "name": "Smoke Test Kitchen Scanner"
            }'
    )"

    assert_status "$STATUS" "204" "rename device"

    # -----------------------------------------------------------------------
    # Verify device rename
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices"
    )"

    assert_status "$STATUS" "200" "list devices after rename"

    if ! jq -e \
        --arg id "$DEVICE_ID" \
        '.[] |
            select(
                .id == $id
                and .name == "Smoke Test Kitchen Scanner"
                and .kind == "scanner"
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: device rename was not persisted"
        exit 1
    fi

    echo "PASS: device rename was persisted"

    # -----------------------------------------------------------------------
    # Issue device credential
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices/$DEVICE_ID/credentials"
    )"

    assert_status "$STATUS" "201" "issue device credential"

    DEVICE_TOKEN="$(jq -r '.token' "$RESPONSE_FILE")"

    if [[ -z "$DEVICE_TOKEN" || "$DEVICE_TOKEN" == "null" ]]; then
        echo "FAIL: device credential issuance did not return a token"
        exit 1
    fi

    echo "PASS: device credential issuance returned a token"

    # -----------------------------------------------------------------------
    # Reject second active credential
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices/$DEVICE_ID/credentials"
    )"

    assert_status "$STATUS" "409" "reject second active device credential"

    # -----------------------------------------------------------------------
    # Rotate device credential
    # -----------------------------------------------------------------------

    OLD_DEVICE_TOKEN="$DEVICE_TOKEN"

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices/$DEVICE_ID/credentials/rotate"
    )"

    assert_status "$STATUS" "200" "rotate device credential"

    DEVICE_TOKEN="$(jq -r '.token' "$RESPONSE_FILE")"

    if [[ -z "$DEVICE_TOKEN" || "$DEVICE_TOKEN" == "null" ]]; then
        echo "FAIL: device credential rotation did not return a token"
        exit 1
    fi

    if [[ "$DEVICE_TOKEN" == "$OLD_DEVICE_TOKEN" ]]; then
        echo "FAIL: device credential rotation returned the previous token"
        exit 1
    fi

    echo "PASS: device credential was rotated"
}

smoke_device_revocation() {
    # -----------------------------------------------------------------------
    # Revoke device
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices/$DEVICE_ID/revoke"
    )"

    assert_status "$STATUS" "204" "revoke device"

    # -----------------------------------------------------------------------
    # Verify revoked device is not returned as active
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices"
    )"

    assert_status "$STATUS" "200" "list devices after revocation"

    if jq -e --arg id "$DEVICE_ID" \
        '.[] | select(.id == $id)' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: revoked device still appears in active device list"
        exit 1
    fi

    echo "PASS: revoked device no longer appears in active device list"

    # -----------------------------------------------------------------------
    # Reject repeated device revocation
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/devices/$DEVICE_ID/revoke"
    )"

    assert_status "$STATUS" "409" "reject already revoked device"
}