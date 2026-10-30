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

nearest_existing_dir() {
  local dir="$1"
  while [[ ! -d "${dir}" && "${dir}" != "/" ]]; do
    dir="$(dirname "${dir}")"
  done
  printf '%s\n' "${dir}"
}

install_binary() {
  local src="$1"
  local dest="${INSTALL_DIR}/${BIN_NAME}"
  if [[ ! -d "${INSTALL_DIR}" ]]; then
    local ancestor
    ancestor="$(nearest_existing_dir "${INSTALL_DIR}")"
    if [[ -w "${ancestor}" ]]; then
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
  PARADISE_VERSION          Release tag to install, for example v0.4.11.
                            Default: the latest GitHub release.
  PARADISE_INSTALL_DIR      Install directory. Default: /usr/local/bin
  PARADISE_ALLOW_UNVERIFIED Set to 1 to install without SHA256SUMS.
                            Not recommended.
  PARADISE_ALLOW_UNSIGNED   Set to 1 only when minisign is not installed.
                            An invalid signature is still refused.
                            A release without SHA256SUMS.minisig is still refused.
  PARADISE_MINISIGN_PUB     Public key file. Default: the key embedded below.
                            Use a copy you saved outside GitHub.

Unknown arguments are refused. A typo such as --unistall does not install.

Published from v0.4.3: Linux aarch64, Linux x86_64, and macOS Apple Silicon.
macOS Intel is not published.
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
  darwin) ;;
  *)
    die "unsupported operating system: ${OS}. Published targets: linux and darwin, aarch64 and x86_64."
    ;;
esac

case "${ARCH}" in
  aarch64|arm64) ASSET_ARCH="aarch64" ;;
  x86_64|amd64) ASSET_ARCH="x86_64" ;;
  *)
    die "unsupported architecture: ${ARCH}. Published targets: aarch64 and x86_64."
    ;;
esac

if [[ "${OS}" == "darwin" && "${ASSET_ARCH}" == "x86_64" ]]; then
  die "macOS Intel is not published. ONNX Runtime has no prebuilt for that target. Use Apple Silicon or Linux."
fi

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

# Do not split the release document on "{". GitHub puts an uploader object
# inside each asset, so the asset name and browser_download_url are not in
# the same brace-delimited chunk. Match the download URL suffix instead.
extract_download_urls() {
  grep -oE '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*"' \
    | sed -E 's/^"browser_download_url"[[:space:]]*:[[:space:]]*"//; s/"$//'
}

extract_exact_field() {
  local field="$1"
  grep -oE "\"${field}\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" \
    | head -1 \
    | sed -E "s/^\"${field}\"[[:space:]]*:[[:space:]]*\"//; s/\"$//" \
    || true
}

extract_exact_asset_url() {
  local name="$1"
  extract_download_urls | awk -v name="${name}" '
    {
      suffix = "/" name
      n = length($0)
      s = length(suffix)
      if (n >= s && substr($0, n - s + 1, s) == suffix) {
        print
        exit
      }
    }
  '
}

list_asset_names() {
  extract_download_urls | awk -F/ '{ print $NF }'
}

# Trust this key, not a key downloaded from the same release. The secret key
# is not stored in the GitHub account.
EMBEDDED_MINISIGN_PUB='RWQlxz3WKJ4Yidsn4mlE0fVRE9ZRcayJPfNw2nmBSw/7VVn/wvWLchjy'

verify_signature() {
  local sums_file="$1"
  local sig_url sig_file pub_file
  if command -v python3 >/dev/null 2>&1; then
    sig_url="$(printf '%s' "${API_JSON}" | parse_release_json "SHA256SUMS.minisig")"
  else
    sig_url="$(printf '%s\n' "${API_JSON}" | extract_exact_asset_url "SHA256SUMS.minisig" || true)"
  fi
  if [[ -z "${sig_url}" ]]; then
    die "this release has no SHA256SUMS.minisig. Refusing to install an unsigned checksum file."
  fi
  sig_file="$(mktemp)"
  TMPFILES+=("${sig_file}")
  info "Downloading SHA256SUMS.minisig"
  curl -fsSL --retry 2 --retry-delay 1 "${sig_url}" -o "${sig_file}"
  if ! command -v minisign >/dev/null 2>&1; then
    if [[ "${PARADISE_ALLOW_UNSIGNED:-0}" == "1" ]]; then
      info "warning: minisign is not installed, so the signature was not checked because PARADISE_ALLOW_UNSIGNED=1."
      return 0
    fi
    die "minisign is required to verify SHA256SUMS. Install minisign, or set PARADISE_ALLOW_UNSIGNED=1 to skip the check."
  fi
  pub_file="$(mktemp)"
  TMPFILES+=("${pub_file}")
  if [[ -n "${PARADISE_MINISIGN_PUB:-}" ]]; then
    if [[ ! -f "${PARADISE_MINISIGN_PUB}" ]]; then
      die "PARADISE_MINISIGN_PUB is set but is not a file: ${PARADISE_MINISIGN_PUB}"
    fi
    cp "${PARADISE_MINISIGN_PUB}" "${pub_file}"
  else
    printf '%s\n%s\n' 'untrusted comment: minisign public key' "${EMBEDDED_MINISIGN_PUB}" >"${pub_file}"
  fi
  if ! minisign -V -p "${pub_file}" -m "${sums_file}" -x "${sig_file}" >/dev/null; then
    die "minisign rejected SHA256SUMS. Not installed."
  fi
  info "minisign signature verified."
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

if [[ "${LATEST}" == "v0.4.0" || "${LATEST}" == "v0.4.1" || "${LATEST}" == "v0.4.4" || "${LATEST}" == "v0.4.5" ]]; then
  die "${LATEST} is withdrawn. Install v0.4.11 or later."
fi

if [[ -z "${DOWNLOAD_URL}" ]]; then
  info "No binary named ${ASSET_NAME} in ${LATEST}."
  info "Assets in this release:"
  if command -v python3 >/dev/null 2>&1; then
    printf '%s' "${API_JSON}" | parse_release_json names || true
  else
    printf '%s\n' "${API_JSON}" | list_asset_names || true
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
  verify_signature "${TMP_SUMS}"
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
