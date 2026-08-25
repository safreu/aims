smoke_qr() {
    # -----------------------------------------------------------------------
    # Create increase QR action
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/qr" \
            -H "Content-Type: application/json" \
            -d "{
                \"item_id\": \"$INVENTORY_ITEM_ID\",
                \"kind\": \"increase\",
                \"amount\": 2
            }"
    )"

    assert_status "$STATUS" "201" "create increase QR action"

    INCREASE_QR_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ -z "$INCREASE_QR_ID" || "$INCREASE_QR_ID" == "null" ]]; then
        echo "FAIL: QR action creation did not return an id"
        exit 1
    fi

    echo "PASS: increase QR action creation returned an id"

    # -----------------------------------------------------------------------
    # Create decrease QR action
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/qr" \
            -H "Content-Type: application/json" \
            -d "{
                \"item_id\": \"$INVENTORY_ITEM_ID\",
                \"kind\": \"decrease\",
                \"amount\": 1
            }"
    )"

    assert_status "$STATUS" "201" "create decrease QR action"

    DECREASE_QR_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ -z "$DECREASE_QR_ID" || "$DECREASE_QR_ID" == "null" ]]; then
        echo "FAIL: decrease QR action creation did not return an id"
        exit 1
    fi

    echo "PASS: decrease QR action creation returned an id"

    # -----------------------------------------------------------------------
    # List QR actions
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/qr"
    )"

    assert_status "$STATUS" "200" "list QR actions"

    if ! jq -e \
        --arg id "$INCREASE_QR_ID" \
        --arg item_id "$INVENTORY_ITEM_ID" \
        '.[] |
            select(
                .id == $id
                and .item_id == $item_id
                and .kind == "increase"
                and .amount == 2
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: increase QR action does not appear correctly in QR list"
        exit 1
    fi

    echo "PASS: increase QR action appears in QR list"

    if ! jq -e \
        --arg id "$DECREASE_QR_ID" \
        --arg item_id "$INVENTORY_ITEM_ID" \
        '.[] |
            select(
                .id == $id
                and .item_id == $item_id
                and .kind == "decrease"
                and .amount == 1
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: decrease QR action does not appear correctly in QR list"
        exit 1
    fi

    echo "PASS: decrease QR action appears in QR list"

    # -----------------------------------------------------------------------
    # Invalid device token is rejected
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -X POST \
            "$BASE_URL/api/v1/device/qr/$INCREASE_QR_ID/execute" \
            -H "Authorization: Bearer invalid-device-token"
    )"

    assert_status "$STATUS" "401" "reject invalid device token for QR execution"

    # -----------------------------------------------------------------------
    # Execute increase QR action
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -X POST \
            "$BASE_URL/api/v1/device/qr/$INCREASE_QR_ID/execute" \
            -H "Authorization: Bearer $DEVICE_TOKEN"
    )"

    assert_status "$STATUS" "204" "execute increase QR action"

    # -----------------------------------------------------------------------
    # Verify QR increase changed stock from 0 to 2
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "200" "get inventory item after QR increase"

    CURRENT_STOCK="$(jq -r '.current_stock' "$RESPONSE_FILE")"

    if [[ "$CURRENT_STOCK" != "2" ]]; then
        echo "FAIL: QR increase produced unexpected stock"
        echo "  expected: 2"
        echo "  actual:   $CURRENT_STOCK"
        exit 1
    fi

    echo "PASS: QR increase changed stock from 0 to 2"

    # -----------------------------------------------------------------------
    # Execute decrease QR action
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -X POST \
            "$BASE_URL/api/v1/device/qr/$DECREASE_QR_ID/execute" \
            -H "Authorization: Bearer $DEVICE_TOKEN"
    )"

    assert_status "$STATUS" "204" "execute decrease QR action"

    # -----------------------------------------------------------------------
    # Verify QR decrease changed stock from 2 to 1
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "200" "get inventory item after QR decrease"

    CURRENT_STOCK="$(jq -r '.current_stock' "$RESPONSE_FILE")"

    if [[ "$CURRENT_STOCK" != "1" ]]; then
        echo "FAIL: QR decrease produced unexpected stock"
        echo "  expected: 1"
        echo "  actual:   $CURRENT_STOCK"
        exit 1
    fi

    echo "PASS: QR decrease changed stock from 2 to 1"

    # -----------------------------------------------------------------------
    # Verify QR stock history
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID/history"
    )"

    assert_status "$STATUS" "200" "list stock history after QR execution"

    if ! jq -e \
        --arg device_id "$DEVICE_ID" \
        '.[] |
            select(
                .kind == "increase"
                and .amount == 2
                and .stock_before == 0
                and .stock_after == 2
                and .source == "qr"
                and .actor.type == "device"
                and .actor.id == $device_id
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: QR increase history entry is missing or incorrect"
        exit 1
    fi

    echo "PASS: QR increase history records QR source and device actor"

    if ! jq -e \
        --arg device_id "$DEVICE_ID" \
        '.[] |
            select(
                .kind == "decrease"
                and .amount == 1
                and .stock_before == 2
                and .stock_after == 1
                and .source == "qr"
                and .actor.type == "device"
                and .actor.id == $device_id
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: QR decrease history entry is missing or incorrect"
        exit 1
    fi

    echo "PASS: QR decrease history records QR source and device actor"

    # -----------------------------------------------------------------------
    # Revoke increase QR action
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/qr/$INCREASE_QR_ID/revoke"
    )"

    assert_status "$STATUS" "204" "revoke QR action"

    # -----------------------------------------------------------------------
    # Revoked QR action is removed from active list
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/qr"
    )"

    assert_status "$STATUS" "200" "list QR actions after revocation"

    if jq -e \
        --arg id "$INCREASE_QR_ID" \
        '.[] | select(.id == $id)' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: revoked QR action still appears in active QR list"
        exit 1
    fi

    echo "PASS: revoked QR action no longer appears in active QR list"

    if ! jq -e \
        --arg id "$DECREASE_QR_ID" \
        '.[] | select(.id == $id)' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: active decrease QR action disappeared after revoking another QR action"
        exit 1
    fi

    echo "PASS: unrelated active QR action remains active"

    # -----------------------------------------------------------------------
    # Revoked QR action cannot be executed
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -X POST \
            "$BASE_URL/api/v1/device/qr/$INCREASE_QR_ID/execute" \
            -H "Authorization: Bearer $DEVICE_TOKEN"
    )"

    assert_status "$STATUS" "409" "reject revoked QR action"

    # -----------------------------------------------------------------------
    # Repeated QR revocation is rejected
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/qr/$INCREASE_QR_ID/revoke"
    )"

    assert_status "$STATUS" "409" "reject already revoked QR action"
}