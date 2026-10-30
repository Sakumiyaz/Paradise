#!/usr/bin/env bash
# Paradise installer.
# Read this file before running it. Do not pipe it straight into a shell.
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

case "${1:-}" in
  --uninstall)
    remove_binary
    exit 0
    ;;
  --help|-h)
    cat <<EOF
Usage: install.sh [--uninstall]

Environment:
  PARADISE_VERSION          Release tag to install, for example v0.4.2.
                            Default: the latest GitHub release.
  PARADISE_INSTALL_DIR      Install directory. Default: /usr/local/bin
  PARADISE_ALLOW_UNVERIFIED Set to 1 to install without SHA256SUMS.
                            Not recommended.

Unknown arguments are refused. A typo such as --unistall does not install.

Published today: linux/aarch64 only.
This public repository does not contain source code, so a failed download
cannot fall back to compiling.
State is stored in ~/.config/paradise, not in the current directory and not in /tmp.
EOF
    exit 0
    ;;
  "")
    ;;
  *)
    die "unknown argument: ${1}. Run 'bash install.sh --help'."
    ;;
esac

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

# The Python program is an argument, not a heredoc. A heredoc would consume
# stdin and drop the release JSON piped into this function.
parse_release_json() {
  local needle="$1"
  PARADISE_PARSE_NEEDLE="${needle}" python3 -c '
import json, os, sys
needle = os.environ.get("PARADISE_PARSE_NEEDLE", "")
try:
    data = json.load(sys.stdin)
except json.JSONDecodeError as exc:
    print(f"invalid release json: {exc}", file=sys.stderr)
    raise SystemExit(2)
if not isinstance(data, dict):
    print("release json was not an object", file=sys.stderr)
    raise SystemExit(2)
if needle == "tag":
    print(data.get("tag_name") or "")
    raise SystemExit(0)
if needle == "names":
    print("\n".join((a.get("name") or "") for a in (data.get("assets") or [])))
    raise SystemExit(0)
for asset in data.get("assets") or []:
    if asset.get("name") == needle:
        print(asset.get("browser_download_url") or "")
        raise SystemExit(0)
print("")
'
}

extract_exact_field() {
  local field="$1"
  awk -v field="${field}" '
    BEGIN { found = 0 }
    {
      pattern = "\"" field "\"[[:space:]]*:[[:space:]]*\"[^\"]*\""
      if (match($0, pattern)) {
        value = substr($0, RSTART, RLENGTH)
        sub("^.*\"" field "\"[[:space:]]*:[[:space:]]*\"", "", value)
        sub("\"$", "", value)
        print value
        found = 1
        exit
      }
    }
    END { if (!found) exit 0 }
  '
}

extract_exact_asset_url() {
  local name="$1"
  awk -v name="${name}" '
    BEGIN { RS = "\\{" ; found = 0 }
    {
      name_re = "\"name\"[[:space:]]*:[[:space:]]*\"" name "\""
      if (match($0, name_re)) {
        if (match($0, "\"browser_download_url\"[[:space:]]*:[[:space:]]*\"[^\"]*\"")) {
          url = substr($0, RSTART, RLENGTH)
          sub("^.*\"browser_download_url\"[[:space:]]*:[[:space:]]*\"", "", url)
          sub("\"$", "", url)
          print url
          found = 1
          exit
        }
      }
    }
    END { if (!found) exit 0 }
  '
}

if command -v python3 >/dev/null 2>&1; then
  if ! LATEST="$(printf '%s' "${API_JSON}" | parse_release_json tag)"; then
    die "could not parse the GitHub release JSON with python3."
  fi
  if ! DOWNLOAD_URL="$(printf '%s' "${API_JSON}" | parse_release_json "${ASSET_NAME}")"; then
    die "could not parse release assets with python3."
  fi
  if ! SUMS_URL="$(printf '%s' "${API_JSON}" | parse_release_json "SHA256SUMS")"; then
    die "could not parse SHA256SUMS from the release JSON."
  fi
else
  LATEST="$(printf '%s\n' "${API_JSON}" | extract_exact_field tag_name || true)"
  DOWNLOAD_URL="$(printf '%s\n' "${API_JSON}" | extract_exact_asset_url "${ASSET_NAME}" || true)"
  SUMS_URL="$(printf '%s\n' "${API_JSON}" | extract_exact_asset_url "SHA256SUMS" || true)"
fi

if [[ -z "${LATEST}" ]]; then
  die "could not read a release tag from the GitHub API. The response was not a release object (rate limit, missing release, or API error)."
fi

info "Release: ${LATEST}"

if [[ "${LATEST}" == "v0.4.0" ]]; then
  die "v0.4.0 compiled in an operator HTTP endpoint and has been withdrawn. Install v0.4.2 or later."
fi

if [[ -z "${DOWNLOAD_URL}" ]]; then
  info "No binary named ${ASSET_NAME} in ${LATEST}."
  info "Assets in this release:"
  if command -v python3 >/dev/null 2>&1; then
    printf '%s' "${API_JSON}" | parse_release_json names || true
  else
    printf '%s\n' "${API_JSON}" | awk '
      {
        while (match($0, /"name"[[:space:]]*:[[:space:]]*"[^"]*"/)) {
          value = substr($0, RSTART, RLENGTH)
          sub(/^.*"name"[[:space:]]*:[[:space:]]*"/, "", value)
          sub(/"$/, "", value)
          print value
          $0 = substr($0, RSTART + RLENGTH)
        }
      }
    ' || true
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
    die "SHA256SUMS does not contain an exact entry for ${ASSET_NAME}"
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
info "State is kept in \${XDG_CONFIG_HOME:-\$HOME/.config}/paradise (directory 0700, files 0600)."
info "It is not stored in the current directory and not in /tmp."
info "Run 'paradise --help' for commands. Uninstall with: bash install.sh --uninstall"
