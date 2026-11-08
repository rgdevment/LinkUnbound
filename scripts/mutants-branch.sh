#!/usr/bin/env bash
set -uo pipefail

if [ -s branch.diff ]; then
  cargo mutants --in-diff branch.diff "$@"
  exit $?
fi

listed=$(cargo mutants --list "$@") || exit $?
if [ -z "$listed" ]; then
  echo "::error::only the configuration changed, and it names no mutant at all"
  exit 1
fi
echo "only the configuration changed; it names $(grep -c . <<< "$listed") mutants"
