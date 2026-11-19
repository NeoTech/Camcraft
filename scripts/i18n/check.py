#!/usr/bin/env python3
"""Check a CamCraft interface catalogue (#853).

Reports, without building anything:
  - a line that is not a `"key" = "value"` pair (with its number),
  - a duplicate key,
  - an empty translation,
  - a translation identical to its source (allowed for 2D/3D, reported as a note),
  - trailing whitespace inside a value.

Usage:
    python3 scripts/i18n/check.py assets/i18n/zh-CN.toml

Exit code 0 when the file is clean, 1 otherwise. Provenance: our own checker.
"""
import sys

SAME_OK = {"2D", "3D"}
ESCAPES = {'"': '"', "\\": "\\", "n": "\n", "t": "\t"}


def unquote(text):
    """Read a quoted run; return (value, rest). Mirrors the Rust reader."""
    out = []
    i = 0
    while i < len(text):
        ch = text[i]
        if ch == '"':
            return "".join(out), text[i + 1:]
        if ch == "\\" and i + 1 < len(text):
            nxt = text[i + 1]
            if nxt in ESCAPES:
                out.append(ESCAPES[nxt])
                i += 2
                continue
            out.append("\\")
            out.append(nxt)
            i += 2
            continue
        out.append(ch)
        i += 1
    return None, ""


def read(path):
    pairs, errors, notes = {}, [], []
    with open(path, encoding="utf-8") as fh:
        for number, raw in enumerate(fh, 1):
            line = raw.rstrip("\n")
            text = line.strip()
            if not text or text.startswith("#"):
                continue
            if not text.startswith('"'):
                errors.append(f"line {number}: a key must start with a quote")
                continue
            key, rest = unquote(text[1:])
            if key is None:
                errors.append(f"line {number}: unterminated key")
                continue
            rest = rest.lstrip()
            if not rest.startswith("="):
                errors.append(f"line {number}: expected `=` after the key")
                continue
            rest = rest[1:].lstrip()
            if not rest.startswith('"'):
                errors.append(f"line {number}: the translation must start with a quote")
                continue
            value, rest = unquote(rest[1:])
            if value is None:
                errors.append(f"line {number}: unterminated translation")
                continue
            tail = rest.strip()
            if tail and not tail.startswith("#"):
                errors.append(f"line {number}: trailing text after the pair")
                continue
            if not key:
                errors.append(f"line {number}: the key is empty")
                continue
            if not value:
                errors.append(f"line {number}: the translation is empty: {key}")
                continue
            if value != value.rstrip():
                notes.append(f"line {number}: trailing space in {key!r}")
            if value == key and key not in SAME_OK:
                notes.append(f"line {number}: {key!r} is its own translation")
            if key in pairs:
                errors.append(f"line {number}: duplicate key {key!r}")
            pairs[key] = value
    return pairs, errors, notes


def main():
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    pairs, errors, notes = read(sys.argv[1])
    for n in notes:
        print(f"note: {n}")
    for e in errors:
        print(f"error: {e}")
    print(f"{sys.argv[1]}: {len(pairs)} strings, {len(errors)} error(s), {len(notes)} note(s)")
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
