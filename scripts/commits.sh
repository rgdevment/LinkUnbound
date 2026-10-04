#!/usr/bin/env bash
set -uo pipefail

shape='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-z0-9._-]+\))?!?: .+'
most=90
status=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
}

# Strict is for a title, which reaches main: only a revert keeps its shape there, and it is measured.
weighed() {
  local who=$1 said=$2 strict=${3:-}
  said=$(printf '%s' "$said" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
  case $said in
    *$'\n'*)
      amiss "$who runs over more than one line"
      printf '  %s\n' "$said"
      return
      ;;
  esac
  if [ -z "$strict" ]; then
    case $said in
      "Merge "* | 'Revert "'* | "fixup! "* | "squash! "* | "amend! "*)
        return
        ;;
    esac
  fi
  case $said in
    'Revert "'*) ;;
    *)
      if ! printf '%s' "$said" | grep -qE "$shape"; then
        amiss "$who does not follow conventional commits"
        printf '  %s\n' "$said"
        return
      fi
      ;;
  esac
  # Bytes minus UTF-8 continuation bytes: ${#said} counts bytes when a GUI client spawns git without a locale.
  local long
  long=$(printf '%s' "$said" | LC_ALL=C tr -d '\200-\277' | wc -c | tr -d ' ')
  if [ "$long" -gt "$most" ]; then
    amiss "$who is $long characters, keep it under $most"
    printf '  %s\n' "$said"
  fi
}

said_so() {
  if [ "$status" -eq 1 ]; then
    echo ""
    echo "Expected: type(optional-scope): description"
    echo "Types:    feat fix docs style refactor perf test build ci chore revert"
    echo "Example:  fix(picker): a link from Teams opens in the browser its rule names"
  fi
  exit $status
}

if [ "${1:-}" = "--subject" ] || [ "${1:-}" = "--title" ]; then
  [ -n "${2:-}" ] || { echo "usage: commits.sh --subject|--title <text>"; exit 2; }
  weighed "the subject" "$2" "$([ "$1" = "--title" ] && echo strict)"
  [ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok the subject is well formed\n'
  said_so
fi

strict=
if [ "${1:-}" = "--landed" ]; then
  strict=strict
  shift
fi
range=${1:-}
[ -n "$range" ] || { echo "usage: commits.sh [--landed] <range> | --subject <text> | --title <text>"; exit 2; }

listed=$(git rev-list --no-merges "$range" 2>&1) || {
  printf '%s\n' "$listed"
  amiss "the commits between $range could not be listed, so no subject was looked at"
  exit 1
}

seen=0
while read -r sha; do
  [ -n "$sha" ] || continue
  seen=$((seen + 1))
  weighed "${sha:0:8}" "$(git log -1 --format=%s "$sha")" "$strict"
done <<< "$listed"

[ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok %s commit subject(s) well formed\n' "$seen"
said_so
