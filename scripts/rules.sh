#!/usr/bin/env bash
set -uo pipefail

status=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
}

went_well() {
  [ -n "${GITHUB_ACTIONS:-}" ] || printf 'ok %s\n' "$1"
}

# grep answers 1 when it finds nothing and 2 when it could not look; only 1 is a pass.
looked_through() {
  case $2 in
    0) amiss "$1"; return 1 ;;
    1) return 0 ;;
    *) amiss "$3"; return 1 ;;
  esac
}

no_prose_blocks() {
  local found
  found=$(find crates/*/src app/src-tauri/src -name '*.rs' -print0 \
    | xargs -0 awk '
        FNR == 1 { run = 0 }
        /^[[:space:]]*\/\/\// { run = 0; next }
        /^[[:space:]]*\/\// && !/\/\/!|TODO|FIXME|SAFETY|noqa|https?:\/\// { run++; if (run == 4) print FILENAME ":" FNR; next }
        { run = 0 }')
  if [ -n "$found" ]; then
    printf '%s\n' "$found"
    amiss "four lines of comment is prose; that belongs in the planning notes"
  else
    went_well "no prose blocks in the code"
  fi
}

nothing_past_what_a_person_holds() {
  local measured verdict
  measured=$(find crates/*/src app/src app/src-tauri/src \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) \
    | grep -vE 'node_modules|/tests/|\.test\.|i18n\.ts' \
    | sort | tr '\n' '\0' | xargs -0 wc -l | awk '$2 != "total" { print $1, $2 }')
  if [ -z "$measured" ]; then
    amiss "no source file was found to measure, so the ceiling was never looked at"
    return
  fi
  verdict=$(awk -v ceiling=1500 '
      FNR == NR { sub(/\r$/, ""); if ($2 != "") kept[$2] = $1; next }
      {
        seen[$2] = 1
        if (!($2 in kept)) {
          if ($1 > ceiling) printf "%s is %d lines of code; %d is the ceiling. Split it along a seam, or say why it belongs in .github/oversized.txt\n", $2, $1, ceiling
        } else if ($1 > kept[$2]) {
          printf "%s was already over the ceiling at %d lines and grew to %d. What is above it only shrinks\n", $2, kept[$2], $1
        } else if ($1 < kept[$2]) {
          printf "%s is down to %d lines from %d. Write that into .github/oversized.txt so it cannot grow back\n", $2, $1, kept[$2]
        }
      }
      END { for (at in kept) if (!(at in seen)) printf "%s is in .github/oversized.txt and no longer exists; take the line out\n", at }
    ' .github/oversized.txt <(printf '%s\n' "$measured"))
  if [ -n "$verdict" ]; then
    while IFS= read -r said; do amiss "$said"; done <<< "$verdict"
  else
    went_well "no file grows past what a person can hold"
  fi
}

read_by_a_person() {
  local where=(app/src/i18n.ts crates/linkunbound-core/src/i18n.rs)
  local docs
  if [ ! -f .github/not-this-spanish.txt ]; then
    amiss "the words the Spanish may not use are not here, so nobody looked for them"
    return
  fi
  docs=$(find . -name '*.md' -not -path '*/node_modules/*' -not -path './target/*' | sort)
  grep -niEf .github/not-this-spanish.txt "${where[@]}" $docs
  looked_through \
    "the Spanish a person reads is neutral, with no voseo and nothing peninsular" $? \
    "the Spanish a person reads could not be looked through where it is written" \
    && went_well "the Spanish a person reads"
}

written_in_english() {
  local found named
  found=$(grep -rnE '\b(regla|reglas|navegador|perfil|dominio|origen|ventana) *:' \
    crates/*/src app/src-tauri/src --include='*.rs')
  if [ $? -gt 1 ]; then
    amiss "the source could not be looked through for Spanish identifiers"
    return
  fi
  named=$(printf '%s\n' "$found" | grep -v '"' | grep -v '^$')
  if [ -n "$named" ]; then
    printf '%s\n' "$named"
    amiss "identifiers are English; what a person reads lives in the locales"
  else
    went_well "identifiers in English"
  fi
}

nothing_the_core_prints() {
  grep -rn 'println!\|eprintln!\|print!\|dbg!' crates/linkunbound-core/src --include='*.rs'
  looked_through \
    "linkunbound-core must not print: the picker inherits it as garbage" $? \
    "linkunbound-core could not be looked through for what it prints" \
    && went_well "the core produces no terminal output"
}

nothing_the_core_knows_of_a_platform() {
  grep -rn 'use tauri\|use windows\|use winreg\|use objc2' crates/linkunbound-core/src --include='*.rs'
  looked_through \
    "the core is pure logic; platform code belongs in its own crate" $? \
    "linkunbound-core could not be looked through for platform dependencies" \
    && went_well "the core stays free of platform and UI dependencies"
}

# Tests live inline, below `#[cfg(test)]`; only what is above it ships.
nothing_that_takes_the_picker_down() {
  local found
  found=$(find crates/linkunbound-core/src -name '*.rs' -print0 \
    | xargs -0 awk '
        FNR == 1 { testing = 0 }
        /^#\[cfg\(test\)\]/ { testing = 1 }
        !testing && /\.unwrap\(\)|\.expect\(|panic!\(|unreachable!\(|todo!\(/ { print FILENAME ":" FNR ": " $0 }')
  if [ -n "$found" ]; then
    printf '%s\n' "$found"
    amiss "the core answers, it never stops: a panic reaches the person as a picker that never came"
  else
    went_well "nothing in the core panics"
  fi
}

unsafe_stays_where_it_was_audited() {
  # native.rs is the Win32 surface the picker needs; shop.rs talks to the Store and reaches
  # unsafe only to hand it a window handle and to join the process apartment. The Mac crate
  # reaches it for framework statics, blocks, a class of its own and the window behind a raw
  # handle; its own audit test lists the same files.
  local audited found
  audited="app/src-tauri/src/shop.rs
crates/linkunbound-mac/src/events.rs
crates/linkunbound-mac/src/icons.rs
crates/linkunbound-mac/src/native.rs
crates/linkunbound-mac/src/registration.rs
crates/linkunbound-mac/src/startup.rs
crates/linkunbound-mac/tests/main_thread.rs
crates/linkunbound-win/src/native.rs"
  found=$(grep -rln 'allow(unsafe_code)' crates app/src-tauri/src --include='*.rs' | sort)
  if [ "$found" != "$(printf '%s\n' "$audited" | sort)" ]; then
    printf '%s\n' "$found"
    amiss "unsafe is allowed somewhere new; audit it and update this list"
  else
    went_well "unsafe stays where it was audited"
  fi
}

cd "$(dirname "$0")/.." || exit 2
no_prose_blocks
nothing_past_what_a_person_holds
read_by_a_person
written_in_english
nothing_the_core_prints
nothing_the_core_knows_of_a_platform
nothing_that_takes_the_picker_down
unsafe_stays_where_it_was_audited
exit $status
