#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# The resident is a binary of its own, and the bundler only carries what `externalBin` names —
# with the host triple appended, which is how Tauri tells one platform's sidecar from another's.
target=$(rustc -vV | sed -n 's/^host: //p')
suffix=""
case "$target" in *windows*) suffix=".exe" ;; esac

profile="${1:-debug}"
case "$profile" in
  release) cargo build --release --bin linkunbound-shell ;;
  *) cargo build --bin linkunbound-shell ;;
esac

mkdir -p app/src-tauri/binaries
cp "target/$profile/linkunbound-shell$suffix" \
   "app/src-tauri/binaries/linkunbound-shell-$target$suffix"
echo "app/src-tauri/binaries/linkunbound-shell-$target$suffix"
