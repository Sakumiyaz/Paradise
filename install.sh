#!/usr/bin/env bash
set -Eeuo pipefail

REPO="Sakumiyaz/ParadiseApp"
INSTALL_DIR="${PARADISE_INSTALL_DIR:-/usr/local/bin}"
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

echo "Paradise — Self-Evolving Code Intelligence"
echo "Detected: ${OS} ${ARCH}"
echo ""

# Map architecture
case "${ARCH}" in
  aarch64|arm64) ASSET_ARCH="aarch64" ;;
  x86_64|amd64)  ASSET_ARCH="x86_64" ;;
  *)
    echo "Unsupported architecture: ${ARCH}"
    echo "Paradise is available for aarch64 and x86_64."
    exit 1
    ;;
esac

# Check for latest release
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -1 | sed 's/.*"tag_name": *"//;s/"//')

if [ -z "${LATEST}" ]; then
  echo "No prebuilt binary found. Building from source..."
  echo ""
  echo "Prerequisites: Rust (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh)"
  echo ""
  # Clone the private repo if user has access, otherwise inform.
  echo "Paradise source is private. If you have access:"
  echo "  git clone git@github.com:Sakumiyaz/Paradise.git"
  echo "  cd Paradise && cargo build --release --bin paradise"
  echo "  sudo cp target/release/paradise ${INSTALL_DIR}/paradise"
  exit 1
fi

echo "Latest version: ${LATEST}"

# Find the matching asset
ASSET_NAME="paradise-${ASSET_ARCH}-${OS}"
DOWNLOAD_URL=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep -o "\"browser_download_url\": *\"[^\"]*${ASSET_NAME}[^\"]*\"" | head -1 | sed 's/.*"browser_download_url": *"//;s/"//')

if [ -z "${DOWNLOAD_URL}" ]; then
  echo "No binary found for ${ASSET_NAME}. Available assets:"
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep -o '"name": *"[^"]*"' | sed 's/.*"name": *"//;s/"//'
  exit 1
fi

echo "Downloading: ${DOWNLOAD_URL}"
TMPFILE=$(mktemp)
curl -fsSL "${DOWNLOAD_URL}" -o "${TMPFILE}"
chmod +x "${TMPFILE}"

echo "Installing to: ${INSTALL_DIR}/paradise"
if [ -w "${INSTALL_DIR}" ]; then
  mv "${TMPFILE}" "${INSTALL_DIR}/paradise"
else
  sudo mv "${TMPFILE}" "${INSTALL_DIR}/paradise"
fi

echo ""
echo "✓ Paradise installed to ${INSTALL_DIR}/paradise"
echo ""
echo "Run 'paradise' to start the TUI."
echo "Run 'paradise version' to check the version."
echo "Run 'paradise help' for all commands."