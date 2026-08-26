smoke_shopping() {
    # -----------------------------------------------------------------------
    # List shopping entries
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping"
    )"

    assert_status "$STATUS" "200" "list shopping entries"

    # At this point inventory.sh has left the item at:
    # current_stock = 0
    # reorder_threshold = 3
    #
    # Therefore:
    # shopping_quantity = 3 - 0 + 1 = 4

    if ! jq -e \
        --arg item_id "$INVENTORY_ITEM_ID" \
        '.inventory_entries[] |
            select(
                .item_id == $item_id
                and .name == "Smoke Test Oat Milk"
                and .category == null
                and .quantity == 4
                and .priority == "medium"
                and .note == null
                and .checked == false
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: calculated inventory shopping entry is missing or incorrect"
        exit 1
    fi

    echo "PASS: calculated inventory shopping entry appears correctly"

    # -----------------------------------------------------------------------
    # Set shopping quantity override
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/items/$INVENTORY_ITEM_ID/quantity" \
            -H "Content-Type: application/json" \
            -d '{
                "quantity": 10
            }'
    )"

    assert_status "$STATUS" "204" "set shopping quantity override"

    # -----------------------------------------------------------------------
    # Set shopping note
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/items/$INVENTORY_ITEM_ID/note" \
            -H "Content-Type: application/json" \
            -d '{
                "note": "Smoke Test Note"
            }'
    )"

    assert_status "$STATUS" "204" "set shopping note"

    # -----------------------------------------------------------------------
    # Check inventory shopping entry
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/items/$INVENTORY_ITEM_ID/checked" \
            -H "Content-Type: application/json" \
            -d '{
                "checked": true
            }'
    )"

    assert_status "$STATUS" "204" "check inventory shopping entry"

    # -----------------------------------------------------------------------
    # Verify inventory shopping state
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping"
    )"

    assert_status "$STATUS" "200" "list shopping entries after state changes"

    if ! jq -e \
        --arg item_id "$INVENTORY_ITEM_ID" \
        '.inventory_entries[] |
            select(
                .item_id == $item_id
                and .quantity == 10
                and .note == "Smoke Test Note"
                and .checked == true
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: inventory shopping state changes were not persisted"
        exit 1
    fi

    echo "PASS: inventory shopping state changes were persisted"

    # -----------------------------------------------------------------------
    # Uncheck inventory shopping entry
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/items/$INVENTORY_ITEM_ID/checked" \
            -H "Content-Type: application/json" \
            -d '{
                "checked": false
            }'
    )"

    assert_status "$STATUS" "204" "uncheck inventory shopping entry"

    # -----------------------------------------------------------------------
    # Reject zero shopping quantity
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/items/$INVENTORY_ITEM_ID/quantity" \
            -H "Content-Type: application/json" \
            -d '{
                "quantity": 0
            }'
    )"

    assert_status "$STATUS" "400" "reject zero shopping quantity"

    # -----------------------------------------------------------------------
    # Reject shopping note longer than 50 characters
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/items/$INVENTORY_ITEM_ID/note" \
            -H "Content-Type: application/json" \
            -d '{
                "note": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }'
    )"

    assert_status "$STATUS" "400" "reject shopping note longer than 50 characters"

    # -----------------------------------------------------------------------
    # Create custom shopping entry
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/custom" \
            -H "Content-Type: application/json" \
            -d '{
                "title": "Smoke Test Batteries",
                "quantity": 4,
                "priority": "high",
                "note": "AA batteries"
            }'
    )"

    assert_status "$STATUS" "201" "create custom shopping entry"

    CUSTOM_SHOPPING_ENTRY_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ -z "$CUSTOM_SHOPPING_ENTRY_ID" || "$CUSTOM_SHOPPING_ENTRY_ID" == "null" ]]; then
        echo "FAIL: custom shopping entry creation did not return an id"
        exit 1
    fi

    echo "PASS: custom shopping entry creation returned an id"

    # -----------------------------------------------------------------------
    # Verify custom shopping entry appears in list
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping"
    )"

    assert_status "$STATUS" "200" "list shopping entries after custom entry creation"

    if ! jq -e \
        --arg entry_id "$CUSTOM_SHOPPING_ENTRY_ID" \
        '.custom_entries[] |
            select(
                .id == $entry_id
                and .title == "Smoke Test Batteries"
                and .quantity == 4
                and .priority == "high"
                and .note == "AA batteries"
                and .checked == false
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: custom shopping entry is missing or incorrect"
        exit 1
    fi

    echo "PASS: custom shopping entry appears correctly"

    # -----------------------------------------------------------------------
    # Update custom shopping entry
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/custom/$CUSTOM_SHOPPING_ENTRY_ID" \
            -H "Content-Type: application/json" \
            -d '{
                "title": "Smoke Test Rechargeable Batteries",
                "quantity": 6,
                "priority": "medium",
                "note": "AA or AAA"
            }'
    )"

    assert_status "$STATUS" "204" "update custom shopping entry"

    # -----------------------------------------------------------------------
    # Check custom shopping entry
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/custom/$CUSTOM_SHOPPING_ENTRY_ID/checked" \
            -H "Content-Type: application/json" \
            -d '{
                "checked": true
            }'
    )"

    assert_status "$STATUS" "204" "check custom shopping entry"

    # -----------------------------------------------------------------------
    # Verify custom shopping entry update
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping"
    )"

    assert_status "$STATUS" "200" "list shopping entries after custom entry update"

    if ! jq -e \
        --arg entry_id "$CUSTOM_SHOPPING_ENTRY_ID" \
        '.custom_entries[] |
            select(
                .id == $entry_id
                and .title == "Smoke Test Rechargeable Batteries"
                and .quantity == 6
                and .priority == "medium"
                and .note == "AA or AAA"
                and .checked == true
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: custom shopping entry update was not persisted"
        exit 1
    fi

    echo "PASS: custom shopping entry update was persisted"

    # -----------------------------------------------------------------------
    # Reject custom shopping update without changes
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/custom/$CUSTOM_SHOPPING_ENTRY_ID" \
            -H "Content-Type: application/json" \
            -d '{}'
    )"

    assert_status "$STATUS" "400" "reject custom shopping update without changes"

    # -----------------------------------------------------------------------
    # Reject custom shopping entry with zero quantity
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/custom" \
            -H "Content-Type: application/json" \
            -d '{
                "title": "Invalid Custom Entry",
                "quantity": 0,
                "priority": "default",
                "note": null
            }'
    )"

    assert_status "$STATUS" "400" "reject custom shopping entry with zero quantity"

    # -----------------------------------------------------------------------
    # Reject invalid custom shopping priority
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/custom" \
            -H "Content-Type: application/json" \
            -d '{
                "title": "Invalid Priority Entry",
                "quantity": 1,
                "priority": "urgent",
                "note": null
            }'
    )"

    assert_status "$STATUS" "400" "reject invalid custom shopping priority"

    # -----------------------------------------------------------------------
    # Delete custom shopping entry
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X DELETE \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/custom/$CUSTOM_SHOPPING_ENTRY_ID"
    )"

    assert_status "$STATUS" "204" "delete custom shopping entry"

    # -----------------------------------------------------------------------
    # Verify deleted custom shopping entry is gone
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping"
    )"

    assert_status "$STATUS" "200" "list shopping entries after custom entry deletion"

    if jq -e \
        --arg entry_id "$CUSTOM_SHOPPING_ENTRY_ID" \
        '.custom_entries[] | select(.id == $entry_id)' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: deleted custom shopping entry still appears"
        exit 1
    fi

    echo "PASS: deleted custom shopping entry no longer appears"

    # -----------------------------------------------------------------------
    # Dismiss inventory shopping entry
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X DELETE \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "204" "dismiss inventory shopping entry"

    # -----------------------------------------------------------------------
    # Verify dismissed inventory shopping entry is hidden
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping"
    )"

    assert_status "$STATUS" "200" "list shopping entries after dismissal"

    if jq -e \
        --arg item_id "$INVENTORY_ITEM_ID" \
        '.inventory_entries[] | select(.item_id == $item_id)' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: dismissed inventory shopping entry still appears"
        exit 1
    fi

    echo "PASS: dismissed inventory shopping entry no longer appears"

    # -----------------------------------------------------------------------
    # Reject unauthenticated shopping request
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/shopping"
    )"

    assert_status "$STATUS" "401" "reject unauthenticated shopping request"
}