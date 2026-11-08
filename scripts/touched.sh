#!/usr/bin/env bash
set -euo pipefail

base=${1:-}
flags=(rust picker deps packaging notices crates mac settings window stryker)

everything() {
  printf '%s=true\n' "${flags[@]}"
  exit 0
}

[ -n "$base" ] || everything

from=$(git merge-base "$base" HEAD)
changed=$(git diff --name-only "$from" HEAD)
present=$(git diff --name-only --diff-filter=d "$from" HEAD)

grep -qx 'scripts/touched\.sh' <<< "$changed" && everything

either() { grep -qE "$1" <<< "$2" && echo true || echo false; }

# A path not known to stay out of Rust's way counts as Rust, so a new one runs the whole suite.
outside_rust='^(app/src/|app/(\.gitignore|biome\.json|index\.html|package(-lock)?\.json|stryker\.config\.json|tsconfig(\.node)?\.json|vite\.config\.ts)$|hooks/|packaging/|resources/|(CLA|COMMERCIAL|CONTRIBUTING|PRIVACY|README|SECURITY)\.md$|package(-lock)?\.json$|\.markdownlint-cli2\.jsonc$|\.editorconfig$|\.github/(FUNDING|dependabot)\.yml$|\.github/(not-this-spanish|oversized)\.txt$|\.github/workflows/(bundle|caches|cla|commits|feed|mutants|mutants-sweep|release|rules|update-badge)\.yml$)'
news_the_resident_embeds='app/src/news\.json'
first=$( { grep -x "$news_the_resident_embeds" <<< "$changed"; grep -vE "$outside_rust" <<< "$changed"; } \
  | grep -v '^$' | head -1 || true)
echo "the first path Rust has to answer for: ${first:-none}" >&2

ci='^\.github/workflows/ci\.yml$'
mutants='^(\.github/workflows/mutants\.yml|\.cargo/mutants\.toml)$'
stryker='^(\.github/workflows/mutants\.yml|app/stryker\.config\.json)$'

echo "rust=$([ -n "$first" ] && echo true || echo false)"
echo "picker=$(either "^app/([^/]+|src/.*)$|$ci" "$changed")"
echo "deps=$(either "(^|/)(Cargo\.(toml|lock)|package(-lock)?\.json)$|^deny\.toml$|$ci" "$changed")"
echo "packaging=$(either '^(app/src-tauri/(nsis/|Info\.plist|tauri\.conf\.json|Cargo\.toml|build\.rs)|scripts/sidecar\.sh|Cargo\.toml$|\.github/workflows/bundle\.yml$)' "$present")"
echo "notices=$(either '^(scripts/third-party\.mjs|THIRD-PARTY-(BUNDLED|LICENSES)\.md|scripts/licences/.*|Cargo\.(lock|toml)|.*/Cargo\.toml|app/package(-lock)?\.json|app/src/.*|app/index\.html|app/vite\.config\.ts|\.github/workflows/rules\.yml)$' "$present")"
echo "crates=$(either "^crates/linkunbound-(core|shell)/|$mutants" "$changed")"
echo "mac=$(either "^crates/linkunbound-(mac|core|shell)/|$mutants" "$changed")"
echo "settings=$(either "^app/src-tauri/src/|$mutants" "$changed")"
echo "window=$(either "^app/src/|$stryker" "$changed")"
echo "stryker=$(either "$stryker" "$changed")"
