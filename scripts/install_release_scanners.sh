#!/usr/bin/env bash
set -Eeuo pipefail

if ! command -v go >/dev/null 2>&1; then
    printf 'go is required to install gitleaks and trufflehog\n' >&2
    exit 127
fi

if ! command -v curl >/dev/null 2>&1; then
    printf 'curl is required to install trufflehog release binaries\n' >&2
    exit 127
fi

INSTALL_DIR="$(go env GOPATH)/bin"
mkdir -p "${INSTALL_DIR}"

GOBIN="${INSTALL_DIR}" go install github.com/zricethezav/gitleaks/v8@latest

TRUFFLEHOG_VERSION="3.95.3"
case "$(uname -s)" in
    Linux) TRUFFLEHOG_OS="linux" ;;
    Darwin) TRUFFLEHOG_OS="darwin" ;;
    *)
        printf 'unsupported OS for trufflehog binary install: %s\n' "$(uname -s)" >&2
        exit 1
        ;;
esac
case "$(uname -m)" in
    x86_64 | amd64) TRUFFLEHOG_ARCH="amd64" ;;
    arm64 | aarch64) TRUFFLEHOG_ARCH="arm64" ;;
    *)
        printf 'unsupported architecture for trufflehog binary install: %s\n' "$(uname -m)" >&2
        exit 1
        ;;
esac

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT
TRUFFLEHOG_ARCHIVE="trufflehog_${TRUFFLEHOG_VERSION}_${TRUFFLEHOG_OS}_${TRUFFLEHOG_ARCH}.tar.gz"
TRUFFLEHOG_URL="https://github.com/trufflesecurity/trufflehog/releases/download/v${TRUFFLEHOG_VERSION}/${TRUFFLEHOG_ARCHIVE}"
curl -fsSL "${TRUFFLEHOG_URL}" -o "${TMP_DIR}/${TRUFFLEHOG_ARCHIVE}"
tar -xzf "${TMP_DIR}/${TRUFFLEHOG_ARCHIVE}" -C "${TMP_DIR}" trufflehog
install -m 0755 "${TMP_DIR}/trufflehog" "${INSTALL_DIR}/trufflehog"

if [[ -n "${GITHUB_PATH:-}" ]]; then
    printf '%s\n' "${INSTALL_DIR}" >>"${GITHUB_PATH}"
fi

printf 'installed release scanners into %s\n' "${INSTALL_DIR}"
