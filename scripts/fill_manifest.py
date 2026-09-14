"""Fills the MSIX manifest from the environment.

Not `sed`: a Partner Center publisher is a distinguished name, and it escapes its own commas as
`O=Contoso\\, Inc.` — a backslash on the right-hand side of a substitution disappears without a
word, and what comes out is a package with an identity nobody notices is wrong until the Store
either refuses it or, worse, accepts it as a different application. An ampersand has the same
problem, loudly.
"""

import os
import sys

MARKS = {
    "@IDENTITY@": "MSIX_IDENTITY",
    "@PUBLISHER@": "MSIX_PUBLISHER",
    "@PUBLISHER_DISPLAY@": "MSIX_PUBLISHER_DISPLAY",
    "@VERSION@": "QUAD",
}
FALLBACKS = {"MSIX_PUBLISHER_DISPLAY": "RGDevment"}


def escaped(value: str) -> str:
    """These land in attribute values, where XML reads markup before anything else does."""
    return (
        value.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def filled(body: str, said: dict[str, str]) -> str:
    for mark, value in said.items():
        body = body.replace(mark, escaped(value))
    return body


def main(source: str, target: str) -> int:
    said = {}
    for mark, name in MARKS.items():
        value = os.environ.get(name) or FALLBACKS.get(name, "")
        if not value:
            print(f"::error::{name} is empty, and {mark} has nothing to become")
            return 1
        said[mark] = value

    with open(source, encoding="utf-8") as handle:
        body = filled(handle.read(), said)

    with open(target, "w", encoding="utf-8") as handle:
        handle.write(body)

    print(body)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
