#!/usr/bin/env bash
# Exercise install.sh without touching GitHub or /usr/local/bin.
set -Eeuo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
installer="${root}/install.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

if bash "${installer}" --unistall >"${tmp}/typo.out" 2>"${tmp}/typo.err"; then
  fail "typo --unistall should fail"
fi
grep -q 'unknown argument' "${tmp}/typo.err" || fail "typo did not report unknown argument"

mkdir -p "${tmp}/bin" "${tmp}/payload"
python3 - "${root}/tests/fixtures/github-release-v0.4.2.json" "${tmp}/api.json" <<'PY'
import json, sys
src, dest = sys.argv[1], sys.argv[2]
data = json.load(open(src))
# Keep the real nested uploader objects. Only the download URLs and tag change.
data["tag_name"] = "v9.9.9"
data["name"] = "Paradise v9.9.9 — fixture title must not look like an asset"
sig = {
    "name": "paradise-aarch64-linux.sig",
    "browser_download_url": "https://example.invalid/releases/download/v9.9.9/paradise-aarch64-linux.sig",
    "uploader": {"login": "decoy", "id": 1, "url": "https://api.github.com/users/decoy"},
}
assets = [sig]
for asset in data.get("assets") or []:
    name = asset.get("name")
    asset["browser_download_url"] = f"https://example.invalid/releases/download/v9.9.9/{name}"
    assets.append(asset)
data["assets"] = assets
json.dump(data, open(dest, "w"))
PY

printf 'fake-binary\n' >"${tmp}/payload/paradise-aarch64-linux"
digest="$(sha256sum "${tmp}/payload/paradise-aarch64-linux" | awk '{print $1}')"
printf '%s  paradise-aarch64-linux\n' "${digest}" >"${tmp}/payload/SHA256SUMS"

cat >"${tmp}/bin/curl" <<EOF
#!/bin/sh
url=""
out=""
while [ \$# -gt 0 ]; do
  case "\$1" in
    -o) out="\$2"; shift 2 ;;
    http*) url="\$1"; shift ;;
    *) shift ;;
  esac
done
case "\$url" in
  *.sig|*.sig*) echo "refusing decoy url \$url" >&2; exit 7 ;;
esac
if [ -n "\$out" ]; then
  case "\$url" in
    *SHA256SUMS) cp "${tmp}/payload/SHA256SUMS" "\$out" ;;
    *paradise-aarch64-linux) cp "${tmp}/payload/paradise-aarch64-linux" "\$out" ;;
    *) echo "unexpected download \$url" >&2; exit 8 ;;
  esac
  exit 0
fi
cat "${tmp}/api.json"
EOF
chmod +x "${tmp}/bin/curl"

cat >"${tmp}/bin/uname" <<'EOF'
#!/bin/sh
if [ "$1" = "-m" ]; then echo aarch64; exit 0; fi
if [ "$1" = "-s" ]; then echo Linux; exit 0; fi
echo Linux
EOF
chmod +x "${tmp}/bin/uname"

run_install() {
  local label="$1"
  local path="$2"
  local dest="${tmp}/dest-${label}"
  rm -rf "${dest}"
  if ! PATH="${path}" PARADISE_INSTALL_DIR="${dest}" /bin/bash "${installer}" >"${tmp}/${label}.out" 2>"${tmp}/${label}.err"; then
    echo "---- ${label} stdout ----" >&2
    cat "${tmp}/${label}.out" >&2
    echo "---- ${label} stderr ----" >&2
    cat "${tmp}/${label}.err" >&2
    fail "${label} installer failed"
  fi
  grep -q 'Release: v9.9.9' "${tmp}/${label}.out" || fail "${label} did not read the release tag"
  grep -q 'SHA-256 verified' "${tmp}/${label}.out" || fail "${label} did not verify SHA-256"
  grep -q 'fake-binary' "${dest}/paradise" || fail "${label} installed the wrong file"
  if grep -q 'fixture title must not look like an asset' "${tmp}/${label}.out"; then
    fail "${label} printed the release title"
  fi
}

run_install python "${tmp}/bin:${PATH}"

# Fallback parser: real utilities, but no python3 on PATH.
nopy="${tmp}/nopy"
mkdir -p "${nopy}"
for cmd in /usr/bin/* /bin/*; do
  base="$(basename "${cmd}")"
  if [[ "${base}" == python3 || "${base}" == python3.* || "${base}" == python ]]; then
    continue
  fi
  ln -sfn "${cmd}" "${nopy}/${base}"
done
if PATH="${tmp}/bin:${nopy}" command -v python3 >/dev/null 2>&1; then
  fail "could not hide python3 for the fallback test"
fi
run_install fallback "${tmp}/bin:${nopy}"

# Unmodified GitHub payload: name and browser_download_url are separated by uploader.
awk '
  /^extract_download_urls\(\)/ { capture = 1 }
  capture { print }
  /^list_asset_names\(\)/ { seen = 1 }
  seen && /^}/ { exit }
' "${installer}" >"${tmp}/parsers.sh"
real_url="$(/bin/bash -c 'source "$1"; extract_exact_asset_url paradise-aarch64-linux' bash "${tmp}/parsers.sh" < "${root}/tests/fixtures/github-release-v0.4.2.json")"
case "${real_url}" in
  */paradise-aarch64-linux) ;;
  *) fail "real API payload did not yield the binary URL: ${real_url}" ;;
esac
real_sums="$(/bin/bash -c 'source "$1"; extract_exact_asset_url SHA256SUMS' bash "${tmp}/parsers.sh" < "${root}/tests/fixtures/github-release-v0.4.2.json")"
case "${real_sums}" in
  */SHA256SUMS) ;;
  *) fail "real API payload did not yield SHA256SUMS: ${real_sums}" ;;
esac
real_names="$(/bin/bash -c 'source "$1"; list_asset_names' bash "${tmp}/parsers.sh" < "${root}/tests/fixtures/github-release-v0.4.2.json")"
if printf '%s
' "${real_names}" | grep -q 'installer can install again'; then
  fail "asset list included the release title"
fi
printf '%s
' "${real_names}" | grep -qx 'paradise-aarch64-linux' || fail "asset list missed the binary"
printf '%s
' "${real_names}" | grep -qx 'SHA256SUMS' || fail "asset list missed SHA256SUMS"

echo "install_test ok"
