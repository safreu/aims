smoke_inventory() {
    # -----------------------------------------------------------------------
    # Create category
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/categories" \
            -H "Content-Type: application/json" \
            -d '{
                "name": "Smoke Test Food"
            }'
    )"

    assert_status "$STATUS" "201" "create category"

    CATEGORY_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ -z "$CATEGORY_ID" || "$CATEGORY_ID" == "null" ]]; then
        echo "FAIL: category creation did not return an id"
        exit 1
    fi

    echo "PASS: category creation returned an id"

    # -----------------------------------------------------------------------
    # List categories
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/categories"
    )"

    assert_status "$STATUS" "200" "list categories"

    if ! jq -e --arg id "$CATEGORY_ID" \
        '.[] | select(.id == $id and .name == "Smoke Test Food")' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: created category was not returned by category list"
        exit 1
    fi

    echo "PASS: created category appears in category list"

    # -----------------------------------------------------------------------
    # Create inventory item
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items" \
            -H "Content-Type: application/json" \
            -d "{
                \"category_id\": \"$CATEGORY_ID\",
                \"name\": \"Smoke Test Milk\",
                \"current_stock\": 2,
                \"reorder_threshold\": 1,
                \"priority\": \"high\"
            }"
    )"

    assert_status "$STATUS" "201" "create inventory item"

    INVENTORY_ITEM_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ -z "$INVENTORY_ITEM_ID" || "$INVENTORY_ITEM_ID" == "null" ]]; then
        echo "FAIL: inventory item creation did not return an id"
        exit 1
    fi

    echo "PASS: inventory item creation returned an id"

    # -----------------------------------------------------------------------
    # List inventory items
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items"
    )"

    assert_status "$STATUS" "200" "list inventory items"

    if ! jq -e \
        --arg item_id "$INVENTORY_ITEM_ID" \
        --arg category_id "$CATEGORY_ID" \
        '.[] |
            select(
                .id == $item_id
                and .name == "Smoke Test Milk"
                and .category.id == $category_id
                and .category.name == "Smoke Test Food"
                and .current_stock == 2
                and .reorder_threshold == 1
                and .priority == "high"
                and .shopping_quantity == 0
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: created inventory item was not returned correctly by inventory list"
        exit 1
    fi

    echo "PASS: created inventory item appears correctly in inventory list"

    # -----------------------------------------------------------------------
    # Get inventory item
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "200" "get inventory item"

    RETURNED_ITEM_ID="$(jq -r '.id' "$RESPONSE_FILE")"
    RETURNED_ITEM_NAME="$(jq -r '.name' "$RESPONSE_FILE")"
    RETURNED_CATEGORY_ID="$(jq -r '.category.id' "$RESPONSE_FILE")"

    if [[ "$RETURNED_ITEM_ID" != "$INVENTORY_ITEM_ID" ]]; then
        echo "FAIL: get inventory item returned unexpected item"
        exit 1
    fi

    if [[ "$RETURNED_ITEM_NAME" != "Smoke Test Milk" ]]; then
        echo "FAIL: get inventory item returned unexpected name"
        echo "  expected: Smoke Test Milk"
        echo "  actual:   $RETURNED_ITEM_NAME"
        exit 1
    fi

    if [[ "$RETURNED_CATEGORY_ID" != "$CATEGORY_ID" ]]; then
        echo "FAIL: get inventory item returned unexpected category"
        exit 1
    fi

    echo "PASS: get inventory item returned correct item"

    # -----------------------------------------------------------------------
    # Update inventory item
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PATCH \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID" \
            -H "Content-Type: application/json" \
            -d '{
                "name": "Smoke Test Oat Milk",
                "reorder_threshold": 3,
                "priority": "medium"
            }'
    )"

    assert_status "$STATUS" "204" "update inventory item"

    # -----------------------------------------------------------------------
    # Verify inventory item update was persisted
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "200" "get updated inventory item"

    UPDATED_NAME="$(jq -r '.name' "$RESPONSE_FILE")"
    UPDATED_REORDER_THRESHOLD="$(jq -r '.reorder_threshold' "$RESPONSE_FILE")"
    UPDATED_PRIORITY="$(jq -r '.priority' "$RESPONSE_FILE")"
    UPDATED_CATEGORY_ID="$(jq -r '.category.id' "$RESPONSE_FILE")"

    if [[ "$UPDATED_NAME" != "Smoke Test Oat Milk" ]]; then
        echo "FAIL: inventory item name update was not persisted"
        echo "  expected: Smoke Test Oat Milk"
        echo "  actual:   $UPDATED_NAME"
        exit 1
    fi

    if [[ "$UPDATED_REORDER_THRESHOLD" != "3" ]]; then
        echo "FAIL: inventory item reorder threshold update was not persisted"
        echo "  expected: 3"
        echo "  actual:   $UPDATED_REORDER_THRESHOLD"
        exit 1
    fi

    if [[ "$UPDATED_PRIORITY" != "medium" ]]; then
        echo "FAIL: inventory item priority update was not persisted"
        echo "  expected: medium"
        echo "  actual:   $UPDATED_PRIORITY"
        exit 1
    fi

    if [[ "$UPDATED_CATEGORY_ID" != "$CATEGORY_ID" ]]; then
        echo "FAIL: omitted category was unexpectedly changed"
        exit 1
    fi

    echo "PASS: inventory item update was persisted"

    # -----------------------------------------------------------------------
    # Archive inventory item
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID/archive"
    )"

    assert_status "$STATUS" "204" "archive inventory item"

    # -----------------------------------------------------------------------
    # Verify archived item is not returned by active item endpoint
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "404" "archived inventory item is hidden from active item endpoint"

    # -----------------------------------------------------------------------
    # Verify archived item is not returned by active inventory list
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items"
    )"

    assert_status "$STATUS" "200" "list active inventory items after archiving"

    if jq -e --arg id "$INVENTORY_ITEM_ID" \
        '.[] | select(.id == $id)' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: archived inventory item still appears in active inventory list"
        exit 1
    fi

    echo "PASS: archived inventory item is hidden from active inventory list"

    # -----------------------------------------------------------------------
    # Restore inventory item
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID/restore"
    )"

    assert_status "$STATUS" "204" "restore inventory item"

    # -----------------------------------------------------------------------
    # Verify restored item is active again
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "200" "get restored inventory item"

    RESTORED_ITEM_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ "$RESTORED_ITEM_ID" != "$INVENTORY_ITEM_ID" ]]; then
        echo "FAIL: restored inventory item was not returned correctly"
        exit 1
    fi

    echo "PASS: restored inventory item is active again"

    # -----------------------------------------------------------------------
    # Increase inventory stock
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID/increase" \
            -H "Content-Type: application/json" \
            -d '{
                "amount": 3
            }'
    )"

    assert_status "$STATUS" "204" "increase inventory stock"

    # -----------------------------------------------------------------------
    # Verify increased stock
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "200" "get inventory item after stock increase"

    CURRENT_STOCK="$(jq -r '.current_stock' "$RESPONSE_FILE")"

    if [[ "$CURRENT_STOCK" != "5" ]]; then
        echo "FAIL: inventory stock increase was not persisted"
        echo "  expected: 5"
        echo "  actual:   $CURRENT_STOCK"
        exit 1
    fi

    echo "PASS: inventory stock increase was persisted"

    # -----------------------------------------------------------------------
    # Decrease inventory stock
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X POST \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID/decrease" \
            -H "Content-Type: application/json" \
            -d '{
                "amount": 2
            }'
    )"

    assert_status "$STATUS" "204" "decrease inventory stock"

    # -----------------------------------------------------------------------
    # Verify decreased stock
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "200" "get inventory item after stock decrease"

    CURRENT_STOCK="$(jq -r '.current_stock' "$RESPONSE_FILE")"

    if [[ "$CURRENT_STOCK" != "3" ]]; then
        echo "FAIL: inventory stock decrease was not persisted"
        echo "  expected: 3"
        echo "  actual:   $CURRENT_STOCK"
        exit 1
    fi

    echo "PASS: inventory stock decrease was persisted"

    # -----------------------------------------------------------------------
    # Set inventory stock
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X PUT \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID/stock" \
            -H "Content-Type: application/json" \
            -d '{
                "stock": 0
            }'
    )"

    assert_status "$STATUS" "204" "set inventory stock"

    # -----------------------------------------------------------------------
    # Verify stock can be set to zero
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID"
    )"

    assert_status "$STATUS" "200" "get inventory item after setting stock"

    CURRENT_STOCK="$(jq -r '.current_stock' "$RESPONSE_FILE")"

    if [[ "$CURRENT_STOCK" != "0" ]]; then
        echo "FAIL: inventory stock set operation was not persisted"
        echo "  expected: 0"
        echo "  actual:   $CURRENT_STOCK"
        exit 1
    fi

    echo "PASS: inventory stock can be set to zero"

    # -----------------------------------------------------------------------
    # List inventory stock history
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items/$INVENTORY_ITEM_ID/history"
    )"

    assert_status "$STATUS" "200" "list inventory stock history"

    HISTORY_COUNT="$(jq 'length' "$RESPONSE_FILE")"

    if [[ "$HISTORY_COUNT" -lt 3 ]]; then
        echo "FAIL: expected at least 3 stock history entries"
        echo "  actual: $HISTORY_COUNT"
        exit 1
    fi

    echo "PASS: stock history contains expected entries"

    # -----------------------------------------------------------------------
    # Verify newest history entry is the stock set operation
    # -----------------------------------------------------------------------

    LATEST_KIND="$(jq -r '.[0].kind' "$RESPONSE_FILE")"
    LATEST_AMOUNT="$(jq -r '.[0].amount' "$RESPONSE_FILE")"
    LATEST_STOCK_BEFORE="$(jq -r '.[0].stock_before' "$RESPONSE_FILE")"
    LATEST_STOCK_AFTER="$(jq -r '.[0].stock_after' "$RESPONSE_FILE")"
    LATEST_ACTOR_TYPE="$(jq -r '.[0].actor.type' "$RESPONSE_FILE")"
    LATEST_ACTOR_ID="$(jq -r '.[0].actor.id' "$RESPONSE_FILE")"

    if [[ "$LATEST_KIND" != "set" ]]; then
        echo "FAIL: latest stock history entry has unexpected kind"
        echo "  expected: set"
        echo "  actual:   $LATEST_KIND"
        exit 1
    fi

    if [[ "$LATEST_AMOUNT" != "null" ]]; then
        echo "FAIL: set stock history entry should have null amount"
        echo "  actual: $LATEST_AMOUNT"
        exit 1
    fi

    if [[ "$LATEST_STOCK_BEFORE" != "3" ]]; then
        echo "FAIL: set stock history entry has unexpected stock_before"
        echo "  expected: 3"
        echo "  actual:   $LATEST_STOCK_BEFORE"
        exit 1
    fi

    if [[ "$LATEST_STOCK_AFTER" != "0" ]]; then
        echo "FAIL: set stock history entry has unexpected stock_after"
        echo "  expected: 0"
        echo "  actual:   $LATEST_STOCK_AFTER"
        exit 1
    fi

    if [[ "$LATEST_ACTOR_TYPE" != "user" ]]; then
        echo "FAIL: stock history entry has unexpected actor type"
        echo "  expected: user"
        echo "  actual:   $LATEST_ACTOR_TYPE"
        exit 1
    fi

    if [[ "$LATEST_ACTOR_ID" != "$OWNER_ID" ]]; then
        echo "FAIL: stock history entry has unexpected actor"
        exit 1
    fi

    echo "PASS: latest stock history entry is correct"

    # -----------------------------------------------------------------------
    # Verify increase history entry
    # -----------------------------------------------------------------------

    if ! jq -e \
        '.[] |
            select(
                .kind == "increase"
                and .amount == 3
                and .stock_before == 2
                and .stock_after == 5
                and .source == "manual"
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: increase stock history entry is missing or incorrect"
        exit 1
    fi

    echo "PASS: increase stock history entry is correct"

    # -----------------------------------------------------------------------
    # Verify decrease history entry
    # -----------------------------------------------------------------------

    if ! jq -e \
        '.[] |
            select(
                .kind == "decrease"
                and .amount == 2
                and .stock_before == 5
                and .stock_after == 3
                and .source == "manual"
            )' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: decrease stock history entry is missing or incorrect"
        exit 1
    fi

    echo "PASS: decrease stock history entry is correct"

    # -----------------------------------------------------------------------
    # Delete category
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            -X DELETE \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/categories/$CATEGORY_ID"
    )"

    assert_status "$STATUS" "204" "delete category"

    # -----------------------------------------------------------------------
    # Verify deleted category no longer appears in category list
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/categories"
    )"

    assert_status "$STATUS" "200" "list categories after deletion"

    if jq -e --arg id "$CATEGORY_ID" \
        '.[] | select(.id == $id)' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: deleted category still appears in category list"
        exit 1
    fi

    echo "PASS: deleted category no longer appears in category list"

    # -----------------------------------------------------------------------
    # Verify deleting category did not delete inventory item
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -b "$COOKIE_FILE" \
            "$BASE_URL/api/v1/inventory/$HOUSEHOLD_ID/items"
    )"

    assert_status "$STATUS" "200" "list inventory items after category deletion"

    if ! jq -e \
        --arg id "$INVENTORY_ITEM_ID" \
        '.[] | select(.id == $id and .category == null)' \
        "$RESPONSE_FILE" >/dev/null; then
        echo "FAIL: inventory item was not preserved without category after category deletion"
        exit 1
    fi

    echo "PASS: inventory item remains with no category after category deletion"
}