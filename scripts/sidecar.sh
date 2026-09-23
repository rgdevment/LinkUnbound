#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

host=$(rustc -vV | sed -n 's/^host: //p')
suffix=""
case "$host" in *windows*) suffix=".exe" ;; esac

profile="${1:-debug}"
target="${2:-}"
out="app/src-tauri/binaries"
mkdir -p "$out"

flags=()
case "$profile" in release) flags+=(--release --locked) ;; esac

build() {
  local triple="$1"
  if [ -z "$triple" ]; then
    cargo build ${flags[@]+"${flags[@]}"} --bin linkunbound-shell
    echo "target/$profile/linkunbound-shell$suffix"
  else
    cargo build ${flags[@]+"${flags[@]}"} --bin linkunbound-shell --target "$triple"
    echo "target/$triple/$profile/linkunbound-shell$suffix"
  fi
}

if [ "$target" = "universal-apple-darwin" ]; then
  arm=$(build aarch64-apple-darwin | tail -1)
  intel=$(build x86_64-apple-darwin | tail -1)
  cp "$arm" "$out/linkunbound-shell-aarch64-apple-darwin"
  cp "$intel" "$out/linkunbound-shell-x86_64-apple-darwin"
  lipo -create -output "$out/linkunbound-shell-$target" "$arm" "$intel"
  lipo -archs "$out/linkunbound-shell-$target"
else
  built=$(build "$target" | tail -1)
  cp "$built" "$out/linkunbound-shell-${target:-$host}$suffix"
fi
echo "$out/linkunbound-shell-${target:-$host}$suffix"
