smoke_auth() {
    # -----------------------------------------------------------------------
    # Health
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            "$BASE_URL/api/v1/health"
    )"

    assert_status "$STATUS" "200" "health check"

    # -----------------------------------------------------------------------
    # Register owner
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -X POST "$BASE_URL/api/v1/auth/register" \
            -H "Content-Type: application/json" \
            -d "{
                \"email\": \"$OWNER_EMAIL\",
                \"display_name\": \"Smoke Owner\",
                \"password\": \"$PASSWORD\"
            }"
    )"

    assert_status "$STATUS" "201" "register owner"

    OWNER_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ -z "$OWNER_ID" || "$OWNER_ID" == "null" ]]; then
        echo "FAIL: owner registration did not return an id"
        exit 1
    fi

    # -----------------------------------------------------------------------
    # Register member
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -X POST "$BASE_URL/api/v1/auth/register" \
            -H "Content-Type: application/json" \
            -d "{
                \"email\": \"$MEMBER_EMAIL\",
                \"display_name\": \"Smoke Member\",
                \"password\": \"$PASSWORD\"
            }"
    )"

    assert_status "$STATUS" "201" "register member"

    MEMBER_ID="$(jq -r '.id' "$RESPONSE_FILE")"

    if [[ -z "$MEMBER_ID" || "$MEMBER_ID" == "null" ]]; then
        echo "FAIL: member registration did not return an id"
        exit 1
    fi

    # -----------------------------------------------------------------------
    # Login owner
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            -c "$COOKIE_FILE" \
            -X POST "$BASE_URL/api/v1/auth/login" \
            -H "Content-Type: application/json" \
            -d "{
                \"email\": \"$OWNER_EMAIL\",
                \"password\": \"$PASSWORD\"
            }"
    )"

    assert_status "$STATUS" "200" "login owner"

    # -----------------------------------------------------------------------
    # Protected endpoint without authentication
    # -----------------------------------------------------------------------

    STATUS="$(
        curl -sS \
            -o "$RESPONSE_FILE" \
            -w "%{http_code}" \
            "$BASE_URL/api/v1/households"
    )"

    assert_status "$STATUS" "401" "reject unauthenticated household request"
}