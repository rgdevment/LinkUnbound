#!/usr/bin/env bash
set -euo pipefail

window=${WINDOW:-window.json}
crates=${CRATES:-crates}
badge=${BADGE:-mutants.json}

shards=$(find . -name 'mutants.json' -path '*window*' | sort)
outcomes=$(find "$crates" -name 'outcomes.json' 2>/dev/null | sort)

told() {
  if [ -z "$1" ]; then echo 0; else printf '%s\n' "$1" | wc -l | tr -d ' '; fi
}

if [ "$(told "$shards")" -lt "${WANT_WINDOW:-1}" ] \
  || [ "$(told "$outcomes")" -lt "${WANT_CRATES:-1}" ]; then
  echo "::error::$(told "$shards") of ${WANT_WINDOW:-1} window reports and $(told "$outcomes") \
of ${WANT_CRATES:-1} crate outcomes reached here: a score over half a sweep would be a lie"
  exit 1
fi

# shellcheck disable=SC2086
jq -s '.[0] * {
    files: (map(.files) | add),
    testFiles: (map(.testFiles // {}) | add)
  }' $shards > "$window"

read -r killed survived < <(
  jq -r '
    [.files[].mutants[].status] as $all
    | [($all | map(select(. == "Killed" or . == "Timeout")) | length),
       ($all | map(select(. == "Survived" or . == "NoCoverage")) | length)]
    | @tsv
  ' "$window"
)

caught=0
missed=0
for one in $outcomes; do
  caught=$((caught + $(jq -r '.caught + .timeout' "$one")))
  missed=$((missed + $(jq -r '.missed' "$one")))
done

live=$((killed + caught))
dead=$((survived + missed))
total=$((live + dead))
score=$(awk -v a="$live" -v b="$total" 'BEGIN { printf "%.1f", b ? a * 100 / b : 0 }')

colour=red
awk -v s="$score" 'BEGIN { exit !(s >= 60) }' && colour=orange
awk -v s="$score" 'BEGIN { exit !(s >= 80) }' && colour=green

jq -n --arg m "$score%" --arg c "$colour" \
  '{schemaVersion: 1, label: "mutants", message: $m, color: $c}' > "$badge"

{
  echo "### What the sweep came to"
  echo
  echo "| | caught | survived | score |"
  echo "| --- | --: | --: | --: |"
  echo "| crates | $caught | $missed | $(awk -v a="$caught" -v b="$((caught + missed))" 'BEGIN { printf "%.1f%%", b ? a * 100 / b : 0 }') |"
  echo "| window | $killed | $survived | $(awk -v a="$killed" -v b="$((killed + survived))" 'BEGIN { printf "%.1f%%", b ? a * 100 / b : 0 }') |"
  echo "| **all** | **$live** | **$dead** | **$score%** |"
} >> "${GITHUB_STEP_SUMMARY:-/dev/stdout}"

echo "$score"
