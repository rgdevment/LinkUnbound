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

weighed() {
  local who=$1 said=$2
  said=$(printf '%s' "$said" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
  # The subject an undone commit gets by default keeps its own shape, as a merge does.
  case $said in
    Revert\ \"*) return ;;
  esac
  if ! printf '%s' "$said" | grep -qE "$shape"; then
    amiss "$who does not follow conventional commits"
    printf '  %s\n' "$said"
    return
  fi
  if [ "${#said}" -gt "$most" ]; then
    amiss "$who is ${#said} characters, keep it under $most"
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

if [ "${1:-}" = "--subject" ]; then
  [ -n "${2:-}" ] || { echo "usage: commits.sh --subject <text>"; exit 2; }
  weighed "the subject" "$2"
  [ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok the subject is well formed\n'
  said_so
fi

range=${1:-}
[ -n "$range" ] || { echo "usage: commits.sh <range> | --subject <text>"; exit 2; }

listed=$(git rev-list --no-merges "$range" 2>&1) || {
  printf '%s\n' "$listed"
  amiss "the commits between $range could not be listed, so no subject was looked at"
  exit 1
}

seen=0
while read -r sha; do
  [ -n "$sha" ] || continue
  seen=$((seen + 1))
  weighed "${sha:0:8}" "$(git log -1 --format=%s "$sha")"
done <<< "$listed"

[ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok %s commit subject(s) well formed\n' "$seen"
said_so
