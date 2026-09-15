#!/usr/bin/env bash

set -euo pipefail

readonly AIMS_REPOSITORY="safreu/aims"
readonly AIMS_INSTALL_PATH="/usr/local/bin/aimsctl"

fail() {
    echo "Error: $*" >&2
    exit 1
}

require_root() {
    if [[ "${EUID}" -ne 0 ]]; then
        fail "This installer must be run as root. Try again with sudo."
    fi
}

parse_version() {
    if [[ $# -gt 1 ]]; then
        fail "Usage: $0 [version]"
    fi

    if [[ $# -eq 0 ]]; then
        echo ""
        return
    fi

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

detect_distribution() {
    if [[ ! -r /etc/os-release ]]; then
        fail "Cannot determine Linux distribution: /etc/os-release is missing."
    fi

    # shellcheck disable=SC1091
    source /etc/os-release

    case "${ID:-}" in
        raspbian|debian)
            echo "debian"
            ;;
        ubuntu)
            echo "ubuntu"
            ;;
        fedora)
            echo "fedora"
            ;;
        arch)
            echo "arch"
            ;;
        *)
            fail "Unsupported Linux distribution: ${ID:-unknown}"
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
    check_command install
    check_command mktemp
    check_command grep
}

check_docker() {
    if ! command -v docker >/dev/null 2>&1; then
        fail "Docker is required but is not installed. Install Docker and run this installer again."
    fi

    if ! docker compose version >/dev/null 2>&1; then
        fail "Docker Compose is required but is not available. Install Docker Compose and run this installer again."
    fi
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

install_aimsctl() {
    local architecture="$1"
    local source_directory="$2"

    local asset="aimsctl-linux-${architecture}"

    echo "Installing aimsctl..."

    install \
        -m 0755 \
        "${source_directory}/${asset}" \
        "${AIMS_INSTALL_PATH}"
}

get_installed_aimsctl_version() {
    "${AIMS_INSTALL_PATH}" version
}

verify_installed_aimsctl() {
    local expected_version="$1"
    local actual_version

    actual_version="$(get_installed_aimsctl_version)"

    if [[ -n "$expected_version" && "$actual_version" != "$expected_version" ]]; then
        fail "Installed aimsctl version is ${actual_version}, expected ${expected_version}."
    fi

    echo "aimsctl ${actual_version} installed successfully."
}

run_installer() {
    echo
    echo "Installing Aims..."
    echo

    "${AIMS_INSTALL_PATH}" install
}

main() {
    require_root

    local requested_version
    local architecture
    local distribution
    local temporary_directory
    local installed_version

    requested_version="$(parse_version "$@")"
    architecture="$(detect_architecture)"
    distribution="$(detect_distribution)"

    echo "Aims bootstrap"
    echo
    echo "Architecture: ${architecture}"
    echo "Distribution: ${distribution}"

    check_dependencies
    check_docker

    echo "Docker:       available"

    temporary_directory="$(mktemp -d)"
    trap 'rm -rf "$temporary_directory"' EXIT

    download_aimsctl \
        "$requested_version" \
        "$architecture" \
        "$temporary_directory"

    verify_aimsctl \
        "$architecture" \
        "$temporary_directory"

    install_aimsctl \
        "$architecture" \
        "$temporary_directory"

    verify_installed_aimsctl "$requested_version"

    installed_version="$(get_installed_aimsctl_version)"

    run_installer

    echo
    echo "Aims ${installed_version} bootstrap completed successfully."
}

main "$@"