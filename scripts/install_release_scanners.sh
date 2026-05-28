#!/usr/bin/env bash
set -Eeuo pipefail

if ! command -v curl >/dev/null 2>&1; then
    printf 'curl is required to install release scanner binaries\n' >&2
    exit 127
fi

if ! command -v tar >/dev/null 2>&1; then
    printf 'tar is required to install release scanner binaries\n' >&2
    exit 127
fi

INSTALL_DIR="${PARADISE_SCANNER_INSTALL_DIR:-${GOBIN:-${HOME}/go/bin}}"
mkdir -p "${INSTALL_DIR}"

GITLEAKS_VERSION="8.30.1"
TRUFFLEHOG_VERSION="3.95.3"
case "$(uname -s)" in
    Linux)
        GITLEAKS_OS="linux"
        TRUFFLEHOG_OS="linux"
        ;;
    Darwin)
        GITLEAKS_OS="darwin"
        TRUFFLEHOG_OS="darwin"
        ;;
    *)
        printf 'unsupported OS for scanner binary install: %s\n' "$(uname -s)" >&2
        exit 1
        ;;
esac
case "$(uname -m)" in
    x86_64 | amd64)
        GITLEAKS_ARCH="x64"
        TRUFFLEHOG_ARCH="amd64"
        ;;
    arm64 | aarch64)
        GITLEAKS_ARCH="arm64"
        TRUFFLEHOG_ARCH="arm64"
        ;;
    *)
        printf 'unsupported architecture for scanner binary install: %s\n' "$(uname -m)" >&2
        exit 1
        ;;
esac

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

GITLEAKS_ARCHIVE="gitleaks_${GITLEAKS_VERSION}_${GITLEAKS_OS}_${GITLEAKS_ARCH}.tar.gz"
GITLEAKS_URL="https://github.com/gitleaks/gitleaks/releases/download/v${GITLEAKS_VERSION}/${GITLEAKS_ARCHIVE}"
curl -fsSL "${GITLEAKS_URL}" -o "${TMP_DIR}/${GITLEAKS_ARCHIVE}"
tar -xzf "${TMP_DIR}/${GITLEAKS_ARCHIVE}" -C "${TMP_DIR}" gitleaks
install -m 0755 "${TMP_DIR}/gitleaks" "${INSTALL_DIR}/gitleaks"

TRUFFLEHOG_ARCHIVE="trufflehog_${TRUFFLEHOG_VERSION}_${TRUFFLEHOG_OS}_${TRUFFLEHOG_ARCH}.tar.gz"
TRUFFLEHOG_URL="https://github.com/trufflesecurity/trufflehog/releases/download/v${TRUFFLEHOG_VERSION}/${TRUFFLEHOG_ARCHIVE}"
curl -fsSL "${TRUFFLEHOG_URL}" -o "${TMP_DIR}/${TRUFFLEHOG_ARCHIVE}"
tar -xzf "${TMP_DIR}/${TRUFFLEHOG_ARCHIVE}" -C "${TMP_DIR}" trufflehog
install -m 0755 "${TMP_DIR}/trufflehog" "${INSTALL_DIR}/trufflehog"

if [[ -n "${GITHUB_PATH:-}" ]]; then
    printf '%s\n' "${INSTALL_DIR}" >>"${GITHUB_PATH}"
fi

printf 'installed release scanners into %s\n' "${INSTALL_DIR}"
