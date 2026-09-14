#!/usr/bin/env bash
# The bundler names the settings binary as the executable, because that is the
# crate it builds. Launch Services delivers a link to whatever CFBundleExecutable
# names, and settings cannot open one: it would raise a window and drop the link.
# The resident is what the registration points at on Windows, and this is the
# same choice spelled the way a bundle spells it.
set -euo pipefail

cd "$(dirname "$0")/.."

app="${1:-target/release/bundle/macos/LinkUnbound.app}"
plist="$app/Contents/Info.plist"
resident="linkunbound-shell"

[ -d "$app" ] || { echo "no hay bundle en $app" >&2; exit 1; }
[ -x "$app/Contents/MacOS/$resident" ] || {
  echo "el bundle no lleva $resident: el sidecar no se empaquetó" >&2
  exit 1
}

named=$(plutil -extract CFBundleExecutable raw -o - "$plist")
if [ "$named" = "$resident" ]; then
  echo "ya apunta al residente"
  exit 0
fi

plutil -replace CFBundleExecutable -string "$resident" "$plist"
plutil -lint "$plist" >/dev/null

# A bundle keeps its signature over the plist, so the flip invalidates whatever
# was there. Re-signing is the release job's business; ad-hoc keeps it runnable.
codesign --force --sign - "$app" 2>/dev/null || true

echo "CFBundleExecutable: $named -> $(plutil -extract CFBundleExecutable raw -o - "$plist")"
