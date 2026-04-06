# Design Decisions and Departures from the Original SPL Specification

This document records every deliberate departure from the original Shakespeare Programming Language specification (Hasselström & Åslund, KTH 2001) made in this interpreter, along with the rationale for each decision. It is intended as a permanent reference for contributors and users who need to understand where and why this implementation diverges from the canonical source.

For behaviour that is identical to the original, no entry is made here. For ambiguities resolved by reference to the `zmbc/shakespearelang` Python implementation, see §12 of `SPECIFICATION.md`.

---

## Background

The original SPL specification exists as an informal document bundled with the `spl2c` compiler (sourceforge, 2001). It was never peer-reviewed or formally published. Two decades of independent re-implementation have produced a small family of interpreters that diverge silently on edge cases. This implementation aims to be the reference implementation: fully specified, tested, and documented. Where it departs from the 2001 original, it does so deliberately and records the departure here.

---

## Decisions

---

### D-001 — Numeric Output Concatenation Ambiguity

**Date:** 2026-04  
**Status:** Documented (no fix; spec-faithful behaviour)  
**Affects:** `src/interpreter.rs` (`Op::Output { kind: IoKind::Numeric, … }`), all programs using numeric output

#### Summary

When an SPL program outputs an integer value using `Open your heart`, the value is written directly as a decimal string with no trailing separator. Consecutive numeric outputs are therefore concatenated in the output stream with no delimiter, making the original token boundaries unrecoverable by a reader.

#### Original behaviour

The 2001 `spl2c` specification defines two output operations:

| Sentence | Output |
|---------|--------|
| `Speak your mind.` | The ASCII character whose code point equals the character's value |
| `Open your heart.` | The decimal representation of the character's value |

No separator, newline, or space is inserted between successive `Open your heart` outputs. This is identical to Python's `print(x, end='')` — the spec has no concept of a numeric output delimiter.

#### Consequence

A program that outputs the values `500`, `63`, and `13` in sequence produces the byte stream `50063 13` (with `13` = newline). A downstream reader cannot determine whether the intent was `500 63` or `5006 3` or any other tokenisation. The only safe interpretation requires knowledge of the program's intent.

This ambiguity is **invisible in practice** for the bundled example programs because those programs exclusively output ASCII-range values (0–127). A single-digit or two-digit decimal fits cleanly within one character, and the program alternates between numeric and character output to produce delimited results. `guess.spl` is the sole canonical example that exposes the issue, outputting `500` (no such ASCII printable) and `63` (`?`) as adjacent numeric tokens.

#### Greedy ASCII decoder behaviour

The web UI's "ASCII decode" toggle applies a greedy 3→2→1 digit fallback:

1. Take up to 3 digits; if `0 ≤ value ≤ 127`, emit the character and advance.
2. Otherwise try 2 digits; if `0 ≤ value ≤ 127`, emit the character and advance.
3. Otherwise emit the single digit as a literal character.

This heuristic decodes all standard ASCII-range numeric output correctly but silently mis-decodes out-of-range values (e.g., `500` is tokenised as `50` = `'2'`, then `0` = `'0'`). ASCII decode should be **disabled** when the program outputs values > 127.

#### Current behaviour

This interpreter faithfully replicates the original behaviour. No separator is added. A `--pretty` flag (opt-in newline after each numeric output) has been considered but not implemented, as it would alter program output for any test that matches against exact byte output.

#### References

- Hasselström & Åslund (2001), §4 "Input/Output"  
- `guess.spl` (canonical bundled example, original spl2c distribution)

---

## Changelog

| Version | Date | Entry |
|---------|------|-------|
| 0.1.0 | 2026-04 | D-001: Documented numeric output concatenation ambiguity; no fix (spec-faithful) |
