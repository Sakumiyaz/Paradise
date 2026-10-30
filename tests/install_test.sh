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

mkdir -p "${tmp}/bin" "${tmp}/payload" "${tmp}/keys"
if ! command -v minisign >/dev/null 2>&1; then
  arch="$(uname -m)"
  case "${arch}" in
    aarch64|arm64) mini_arch=aarch64 ;;
    x86_64|amd64) mini_arch=x86_64 ;;
    *) fail "no minisign binary for ${arch}" ;;
  esac
  curl -fsSL -o "${tmp}/minisign.tar.gz" https://github.com/jedisct1/minisign/releases/download/0.12/minisign-0.12-linux.tar.gz
  tar -C "${tmp}" -xzf "${tmp}/minisign.tar.gz"
  install -m 0755 "${tmp}/minisign-linux/${mini_arch}/minisign" "${tmp}/bin/minisign"
else
  ln -s "$(command -v minisign)" "${tmp}/bin/minisign"
fi
printf 'fake-binary\n' >"${tmp}/payload/paradise-aarch64-linux"
digest="$(sha256sum "${tmp}/payload/paradise-aarch64-linux" | awk '{print $1}')"
printf '%s  paradise-aarch64-linux\n' "${digest}" >"${tmp}/payload/SHA256SUMS"
minisign -G -W -p "${tmp}/keys/minisign.pub" -s "${tmp}/keys/minisign.key" >/dev/null
minisign -S -W -s "${tmp}/keys/minisign.key" -m "${tmp}/payload/SHA256SUMS" -x "${tmp}/payload/SHA256SUMS.minisig" >/dev/null
python3 - "${root}/tests/fixtures/github-release-v0.4.2.json" "${tmp}/api.json" <<'PY'
import json, sys
src, dest = sys.argv[1], sys.argv[2]
data = json.load(open(src))
# Keep the real nested uploader objects. Only the download URLs and tag change.
data["tag_name"] = "v9.9.9"
data["name"] = "Paradise v9.9.9 — fixture title must not look like an asset"
extra = []
for name in ("paradise-aarch64-linux.sig", "SHA256SUMS.minisig"):
    extra.append({
        "name": name,
        "browser_download_url": f"https://example.invalid/releases/download/v9.9.9/{name}",
        "uploader": {"login": "decoy", "id": 1, "url": "https://api.github.com/users/decoy"},
    })
assets = extra
for asset in data.get("assets") or []:
    name = asset.get("name")
    asset["browser_download_url"] = f"https://example.invalid/releases/download/v9.9.9/{name}"
    assets.append(asset)
data["assets"] = assets
json.dump(data, open(dest, "w"))
PY

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
  */paradise-aarch64-linux.sig) echo "refusing decoy url \$url" >&2; exit 7 ;;
esac
if [ -n "\$out" ]; then
  case "\$url" in
    */SHA256SUMS.minisig) cp "${tmp}/payload/SHA256SUMS.minisig" "\$out" ;;
    */SHA256SUMS) cp "${tmp}/payload/SHA256SUMS" "\$out" ;;
    */paradise-aarch64-linux) cp "${tmp}/payload/paradise-aarch64-linux" "\$out" ;;
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
  if ! PATH="${path}" PARADISE_INSTALL_DIR="${dest}" PARADISE_MINISIGN_PUB="${tmp}/keys/minisign.pub" /bin/bash "${installer}" >"${tmp}/${label}.out" 2>"${tmp}/${label}.err"; then
    echo "---- ${label} stdout ----" >&2
    cat "${tmp}/${label}.out" >&2
    echo "---- ${label} stderr ----" >&2
    cat "${tmp}/${label}.err" >&2
    fail "${label} installer failed"
  fi
  grep -q 'Release: v9.9.9' "${tmp}/${label}.out" || fail "${label} did not read the release tag"
  grep -q 'SHA-256 verified' "${tmp}/${label}.out" || fail "${label} did not verify SHA-256"
  grep -q 'minisign signature verified' "${tmp}/${label}.out" || fail "${label} did not verify the signature"
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

echo "install_test ok"
