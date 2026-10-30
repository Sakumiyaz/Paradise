#!/usr/bin/env bash
# Paradise installer.
# Review this file before running it. Do not pipe it straight into a shell.
set -Eeuo pipefail

REPO="Sakumiyaz/Paradise"
INSTALL_DIR="${PARADISE_INSTALL_DIR:-/usr/local/bin}"
BIN_NAME="paradise"
VERSION="${PARADISE_VERSION:-}"
ALLOW_UNVERIFIED="${PARADISE_ALLOW_UNVERIFIED:-0}"
TMPFILES=()

cleanup() {
  local f
  for f in "${TMPFILES[@]+"${TMPFILES[@]}"}"; do
    rm -f -- "${f}" 2>/dev/null || true
  done
}
trap cleanup EXIT

die() {
  echo "error: $*" >&2
  exit 1
}

info() {
  echo "$*"
}

need() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

install_binary() {
  local src="$1"
  local dest="${INSTALL_DIR}/${BIN_NAME}"
  if [[ ! -d "${INSTALL_DIR}" ]]; then
    if [[ -w "$(dirname "${INSTALL_DIR}")" ]]; then
      mkdir -p -- "${INSTALL_DIR}"
    else
      sudo mkdir -p -- "${INSTALL_DIR}"
    fi
  fi
  if [[ -w "${INSTALL_DIR}" ]]; then
    install -m 0755 "${src}" "${dest}"
  else
    sudo install -m 0755 -o root -g root "${src}" "${dest}"
  fi
}

remove_binary() {
  local dest="${INSTALL_DIR}/${BIN_NAME}"
  if [[ -w "${INSTALL_DIR}" ]]; then
    rm -f -- "${dest}"
  else
    sudo rm -f -- "${dest}"
  fi
  info "Removed ${dest} if it was installed."
}

if [[ "${1:-}" == "--uninstall" ]]; then
  remove_binary
  exit 0
fi

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<EOF
Usage: install.sh [--uninstall]

Environment:
  PARADISE_VERSION          Release tag to install, for example v0.4.0.
                            Default: the latest GitHub release.
  PARADISE_INSTALL_DIR      Install directory. Default: /usr/local/bin
  PARADISE_ALLOW_UNVERIFIED Set to 1 to install without SHA256SUMS.
                            Not recommended.

Published today: linux/aarch64 only.
This public repository does not contain source code, so a failed download
cannot fall back to compiling.
EOF
  exit 0
fi

need curl
need uname
need install
need mktemp

ARCH="$(uname -m)"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"

info "Paradise installer"
info "Detected: ${OS} ${ARCH}"
info ""

case "${OS}" in
  linux) ;;
  darwin)
    die "macOS is not published yet. Supported today: linux/aarch64. No binary was downloaded."
    ;;
  *)
    die "unsupported operating system: ${OS}. Supported today: linux/aarch64."
    ;;
esac

case "${ARCH}" in
  aarch64|arm64) ASSET_ARCH="aarch64" ;;
  x86_64|amd64)
    die "linux/x86_64 is not published yet. The current release only has paradise-aarch64-linux. No binary was downloaded."
    ;;
  *)
    die "unsupported architecture: ${ARCH}. Supported today: linux/aarch64."
    ;;
esac

ASSET_NAME="paradise-${ASSET_ARCH}-${OS}"

if [[ -n "${VERSION}" ]]; then
  API_URL="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
else
  API_URL="https://api.github.com/repos/${REPO}/releases/latest"
fi

info "Fetching release metadata..."
if ! API_JSON="$(curl -fsSL --retry 2 --retry-delay 1 "${API_URL}")"; then
  die "GitHub API request failed (${API_URL}). Check the network or the unauthenticated rate limit (60 requests/hour). Set PARADISE_VERSION if you already know the tag. This public repository has no source, so the installer will not try to compile."
fi

python_parse() {
  python3 - "$1" <<'PY'
import json, sys
needle = sys.argv[1]
data = json.load(sys.stdin)
if needle == "tag":
    print(data.get("tag_name") or "")
    raise SystemExit(0)
assets = data.get("assets") or []
for asset in assets:
    if asset.get("name") == needle:
        print(asset.get("browser_download_url") or "")
        raise SystemExit(0)
print("")
PY
}

fallback_tag() {
  printf '%s\n' "${API_JSON}" | grep -o '"tag_name": *"[^"]*"' | head -1 | sed 's/.*"tag_name": *"//;s/"//' || true
}

fallback_asset_url() {
  local name="$1"
  printf '%s\n' "${API_JSON}" | grep -o "\"browser_download_url\": *\"[^\"]*${name}[^\"]*\"" | head -1 | sed 's/.*"browser_download_url": *"//;s/"//' || true
}

if command -v python3 >/dev/null 2>&1; then
  LATEST="$(printf '%s' "${API_JSON}" | python_parse tag || true)"
  DOWNLOAD_URL="$(printf '%s' "${API_JSON}" | python_parse "${ASSET_NAME}" || true)"
  SUMS_URL="$(printf '%s' "${API_JSON}" | python_parse "SHA256SUMS" || true)"
else
  LATEST="$(fallback_tag || true)"
  DOWNLOAD_URL="$(fallback_asset_url "${ASSET_NAME}" || true)"
  SUMS_URL="$(fallback_asset_url "SHA256SUMS" || true)"
fi

if [[ -z "${LATEST}" ]]; then
  die "could not read a release tag from the GitHub API. The response was not a release object (rate limit, missing release, or API error)."
fi

info "Release: ${LATEST}"

if [[ "${LATEST}" == "v0.4.0" && "${PARADISE_ALLOW_INSECURE_RELEASE:-0}" != "1" ]]; then
  die "v0.4.0 compiled in an operator HTTP endpoint and is not a safe default install. Publish a rebuilt release, or set PARADISE_ALLOW_INSECURE_RELEASE=1 to install that exact asset anyway."
fi

if [[ -z "${DOWNLOAD_URL}" ]]; then
  info "No binary named ${ASSET_NAME} in ${LATEST}."
  info "Assets in this release:"
  if command -v python3 >/dev/null 2>&1; then
    printf '%s' "${API_JSON}" | python3 -c 'import json,sys; data=json.load(sys.stdin); print("\n".join(a.get("name","") for a in data.get("assets") or []) or "(none)")'
  else
    printf '%s\n' "${API_JSON}" | grep -o '"name": *"[^"]*"' | sed 's/.*"name": *"//;s/"//' || true
  fi
  die "refusing to install a mismatched or missing binary."
fi

TMP_BIN="$(mktemp)"
TMPFILES+=("${TMP_BIN}")
info "Downloading ${ASSET_NAME}"
curl -fsSL --retry 2 --retry-delay 1 "${DOWNLOAD_URL}" -o "${TMP_BIN}"

if [[ -n "${SUMS_URL}" ]]; then
  need sha256sum
  TMP_SUMS="$(mktemp)"
  TMPFILES+=("${TMP_SUMS}")
  info "Downloading SHA256SUMS"
  curl -fsSL --retry 2 --retry-delay 1 "${SUMS_URL}" -o "${TMP_SUMS}"
  expected="$(awk -v name="${ASSET_NAME}" '$2 == name { print $1; exit }' "${TMP_SUMS}")"
  if [[ -z "${expected}" ]]; then
    die "SHA256SUMS does not contain ${ASSET_NAME}"
  fi
  actual="$(sha256sum "${TMP_BIN}" | awk '{print $1}')"
  if [[ "${actual}" != "${expected}" ]]; then
    die "SHA-256 mismatch for ${ASSET_NAME}. expected ${expected}, got ${actual}. Not installed."
  fi
  info "SHA-256 verified."
elif [[ "${ALLOW_UNVERIFIED}" == "1" ]]; then
  info "warning: this release has no SHA256SUMS. Installing because PARADISE_ALLOW_UNVERIFIED=1."
else
  die "this release has no SHA256SUMS asset. Refusing to install an unverified binary, including with sudo. Publish SHA256SUMS, or set PARADISE_ALLOW_UNVERIFIED=1 if you accept the risk."
fi

install_binary "${TMP_BIN}"
info ""
info "Paradise installed to ${INSTALL_DIR}/${BIN_NAME} (mode 0755)."
info "State stays in \${PARADISE_STATE_DIR:-\$PWD/.paradise} and is not sent anywhere unless you configure a provider."
info "Run 'paradise --help' for commands. Uninstall with: bash install.sh --uninstall"
