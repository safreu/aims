#!/usr/bin/env bash

set -euo pipefail

readonly AIMS_REPOSITORY="safreu/aims"

TEMPORARY_DIRECTORY=""

cleanup() {
    if [[ -n "$TEMPORARY_DIRECTORY" && -d "$TEMPORARY_DIRECTORY" ]]; then
        rm -rf "$TEMPORARY_DIRECTORY"
    fi
}

fail() {
    echo "Error: $*" >&2
    exit 1
}

parse_version() {
    local version="$1"

    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        fail "Invalid Aims version: $version"
    fi

    echo "$version"
}

detect_architecture() {
    case "$(uname -m)" in
        x86_64)
            echo "amd64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            fail "Unsupported architecture: $(uname -m)"
            ;;
    esac
}

check_command() {
    local command="$1"

    if ! command -v "$command" >/dev/null 2>&1; then
        fail "'$command' is required but is not installed."
    fi
}

check_dependencies() {
    check_command curl
    check_command sha256sum
    check_command mktemp
    check_command grep
}

download_aimsctl() {
    local version="$1"
    local architecture="$2"
    local destination="$3"

    local asset="aimsctl-linux-${architecture}"
    local base_url

    if [[ -n "$version" ]]; then
        base_url="https://github.com/${AIMS_REPOSITORY}/releases/download/v${version}"

        echo
        echo "Downloading aimsctl ${version}..."
    else
        base_url="https://github.com/${AIMS_REPOSITORY}/releases/latest/download"

        echo
        echo "Downloading latest aimsctl..."
    fi

    curl \
        --fail \
        --location \
        --silent \
        --show-error \
        "${base_url}/${asset}" \
        --output "${destination}/${asset}"

    curl \
        --fail \
        --location \
        --silent \
        --show-error \
        "${base_url}/SHA256SUMS" \
        --output "${destination}/SHA256SUMS"
}

verify_aimsctl() {
    local architecture="$1"
    local directory="$2"

    local asset="aimsctl-linux-${architecture}"

    echo "Verifying download..."

    (
        cd "$directory"

        grep " ${asset}$" SHA256SUMS |
            sha256sum --check -
    )
}

run_installer() {
    local architecture="$1"
    local source_directory="$2"
    shift 2
    
    local asset="aimsctl-linux-${architecture}"
    
    echo
    echo "Installing Aims..."
    echo

    chmod +x "${source_directory}/${asset}"

    "${source_directory}/${asset}" install "$@"
}

main() {
    local requested_version=""
    local architecture

    if [[ $# -gt 0 && "$1" != "--" ]]; then
        requested_version="$(parse_version "$1")"
        shift
    fi

    if [[ $# -gt 0 ]]; then
        if [[ "$1" != "--" ]]; then
            fail "Usage: $0 [version] [-- aimsctl-install-options...]"
        fi

        shift
    fi
    
    architecture="$(detect_architecture)"

    echo "Aims bootstrap"
    echo
    echo "Architecture: ${architecture}"

    check_dependencies

    TEMPORARY_DIRECTORY="$(mktemp -d)"
    trap cleanup EXIT

    download_aimsctl \
        "$requested_version" \
        "$architecture" \
        "$TEMPORARY_DIRECTORY"

    verify_aimsctl \
        "$architecture" \
        "$TEMPORARY_DIRECTORY"

    run_installer \
        "$architecture" \
        "$TEMPORARY_DIRECTORY" \
        "$@"

    echo
    echo "Aims bootstrap completed successfully."
}

main "$@"