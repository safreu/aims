smoke_events() {
    local sse_output
    local sse_pid
    local event_count

    sse_output="$(mktemp)"
    sse_pid=""

    cleanup_sse() {
        if [[ -n "$sse_pid" ]]; then
            kill "$sse_pid" 2>/dev/null || true
            wait "$sse_pid" 2>/dev/null || true
        fi

        rm -f "$sse_output"
    }

    # -----------------------------------------------------------------------
    # Open household event stream
    # -----------------------------------------------------------------------

    curl -sS -N \
        -b "$COOKIE_FILE" \
        "$BASE_URL/api/v1/households/$HOUSEHOLD_ID/events" \
        > "$sse_output" &

    sse_pid=$!

    sleep 1

    if ! kill -0 "$sse_pid" 2>/dev/null; then
        echo "FAIL: household event stream terminated unexpectedly"

        if [[ -s "$sse_output" ]]; then
            echo "SSE output:"
            cat "$sse_output"
            echo
        fi

        cleanup_sse
        exit 1
    fi

    echo "PASS: household event stream opened"

    # -----------------------------------------------------------------------
    # Shopping mutation publishes event
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
                "quantity": 7
            }'
    )"

    assert_status "$STATUS" "204" "trigger shopping mutation for household event"

    for _ in {1..20}; do
        event_count="$(
            grep -c "^event: shopping_list_changed" "$sse_output" || true
        )"

        if (( event_count >= 1 )); then
            break
        fi

        sleep 0.1
    done

    if (( event_count < 1 )); then
        echo "FAIL: shopping mutation did not publish shopping_list_changed"

        if [[ -s "$sse_output" ]]; then
            echo "SSE output:"
            cat "$sse_output"
            echo
        fi

        cleanup_sse
        exit 1
    fi

    echo "PASS: shopping mutation published shopping_list_changed"

    # -----------------------------------------------------------------------
    # Inventory mutation publishes event
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
                "priority": "high"
            }'
    )"

    assert_status "$STATUS" "204" "trigger inventory mutation for household event"

    for _ in {1..20}; do
        event_count="$(
            grep -c "^event: shopping_list_changed" "$sse_output" || true
        )"

        if (( event_count >= 2 )); then
            break
        fi

        sleep 0.1
    done

    if (( event_count < 2 )); then
        echo "FAIL: inventory mutation did not publish shopping_list_changed"

        if [[ -s "$sse_output" ]]; then
            echo "SSE output:"
            cat "$sse_output"
            echo
        fi

        cleanup_sse
        exit 1
    fi

    echo "PASS: inventory mutation published shopping_list_changed"
    cleanup_sse
}