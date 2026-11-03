#!/usr/bin/env bash
set -uo pipefail

kinds='feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert'
shape="^($kinds)(\([a-z0-9._-]+\))?!?: .+"
branched="^($kinds|hotfix|release)/[a-z0-9][a-z0-9._/-]*$"
most=120
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
  # A subject can become the squash title, and GitHub ends that with " (#1234)", so it leaves the room.
  local room=$most
  [ -n "$strict" ] || room=$((most - 8))
  if [ "$long" -gt "$room" ]; then
    amiss "$who is $long characters, keep it under $room"
    printf '  %s\n' "$said"
  fi
}

# A commit is one line; what git wrote for a revert, a fixup or Dependabot is let through.
nothing_under() {
  local author subject body
  IFS=$'\x1f' read -r -d '' author subject body < <(git log -1 -z --format='%an%x1f%s%x1f%b' "$1")
  case $subject in 'Revert "'* | "fixup! "* | "squash! "* | "amend! "*) return ;; esac
  [ "$author" != "dependabot[bot]" ] || return
  if [ -n "$(printf '%s' "$body" | tr -d '[:space:]')" ]; then
    amiss "${1:0:8} carries lines under its subject; a commit is one line"
  fi
}

said_so() {
  if [ "$status" -eq 1 ]; then
    echo ""
    echo "Expected: type(optional-scope): description, on one line with nothing under it"
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

# A branch says what kind of change it carries, the way its commits do; main, a detached head and
# the branches GitHub names for Dependabot say nothing.
if [ "${1:-}" = "--branch" ]; then
  shift
  for name in "$@"; do
    case $name in
      main | HEAD | "" | dependabot/*) continue ;;
    esac
    if ! printf '%s' "$name" | grep -qE "$branched"; then
      amiss "the branch $name is not named kind/what, as in feat/picker-tiles; kind is a commit type, hotfix or release"
    fi
  done
  exit $status
fi

strict=
if [ "${1:-}" = "--landed" ]; then
  strict=strict
  shift
fi
range=${1:-}
[ -n "$range" ] || { echo "usage: commits.sh [--landed] <range> | --subject <text> | --title <text> | --branch <name>..."; exit 2; }

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
  # On main a squash carries the pull request's body, so only a branch's own commits are held to one line.
  [ -n "$strict" ] || nothing_under "$sha"
done <<< "$listed"

[ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok %s commit subject(s) well formed\n' "$seen"
said_so
