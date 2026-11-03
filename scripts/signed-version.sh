#!/usr/bin/env bash
# The version an installer's signature names, as the updater compares it; empty when it names none.
set -euo pipefail

file=$1
signature=$2
key=$3

trusted=$(minisign -Vm "$file" -p "$key" -x "$signature") || exit 1
printf '%s\n' "$trusted" \
  | sed -n 's/^Trusted comment: //p' \
  | tr '\t' '\n' \
  | sed -n 's/^version://p' \
  | sed 's/^v//; s/+.*//'
