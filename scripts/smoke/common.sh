BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"

COOKIE_FILE=""
RESPONSE_FILE=""

RUN_ID=""
OWNER_EMAIL=""
MEMBER_EMAIL=""
PASSWORD=""

setup_smoke_test() {
    COOKIE_FILE="$(mktemp)"
    RESPONSE_FILE="$(mktemp)"

    RUN_ID="$(date +%s)"

    OWNER_EMAIL="smoke-owner-${RUN_ID}@example.com"
    MEMBER_EMAIL="smoke-member-${RUN_ID}@example.com"
    PASSWORD="SuperSecretPassword123!"

    trap cleanup_smoke_test EXIT
}

cleanup_smoke_test() {
    rm -f "$COOKIE_FILE" "$RESPONSE_FILE"
}

assert_status() {
    local actual="$1"
    local expected="$2"
    local description="$3"

    if [[ "$actual" != "$expected" ]]; then
        echo "FAIL: $description"
        echo "  expected: $expected"
        echo "  actual:   $actual"

        if [[ -s "$RESPONSE_FILE" ]]; then
            echo "  response:"
            cat "$RESPONSE_FILE"
            echo
        fi

        exit 1
    fi

    echo "PASS: $description"
}