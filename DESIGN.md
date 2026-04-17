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

---

### D-002 — Vocabulary Provenance: spl2c Semantics over zmbc

**Date:** 2026-04  
**Status:** Deliberate deviation from `zmbc/shakespearelang`  
**Affects:** `src/vocab.rs` (all word lists), `src/parser.rs` (noun-phrase classification)

#### Summary

Where the original `spl2c` (Perl, 2001) and `zmbc/shakespearelang` (Python, ~2015) disagree on the computational role of a specific word, this implementation follows `spl2c`. A reference interpreter should be faithful to the language's origin, not to a later re-implementation.

#### Specific divergences

| Word | zmbc role | spl2c role | spl-rs decision |
|------|-----------|-----------|-----------------|
| `town` | negative noun (−1) | positive noun (+1) | positive — spl2c |
| `wall` | not in list | positive noun (+1) | positive — spl2c |
| `road` | negative noun (−1) | neutral noun (+1) | neutral — spl2c (this was a spl-rs bug, fixed 2026-04) |

The `road` case was initially implemented as negative (following zmbc) and corrected after cross-checking `spl2c` v1.2.1's `neutral_noun.wordlist`.

#### Three-category vocabulary model

The original `spl2c` grammar has three separate adjective categories (`POSITIVE_ADJECTIVE`, `NEUTRAL_ADJECTIVE`, `NEGATIVE_ADJECTIVE`) and three noun categories (`POSITIVE_NOUN`, `NEUTRAL_NOUN`, `NEGATIVE_NOUN`). Many re-implementations collapse these to two categories (positive/negative). This implementation restores the three-category model because:

1. The computational behaviour of neutral adjectives (doubling magnitude) is identical to positive/negative, so no existing program is affected.
2. The three-category distinction is necessary for corpus provenance tracking (SPL-F2 F1 profile): the original word list included neutral nouns/adjectives that have different attestation profiles from positive/negative ones.
3. A reference interpreter should be faithful to the grammar spec, even for distinctions that have no current semantic effect.

#### References

- `spl2c` v1.2.1 wordlist files: `positive_adjective.wordlist`, `neutral_adjective.wordlist`, `negative_adjective.wordlist`, `positive_noun.wordlist`, `neutral_noun.wordlist`, `negative_noun.wordlist`
- `zmbc/shakespearelang` grammar (GitHub)

---

### D-003 — SPL-F2 Ornamental Sentences (Rules O2–O5)

**Date:** 2026-04  
**Status:** Extension (opt-in via `--f2` / `ParseMode::F2`; off by default)  
**Affects:** `src/parser.rs` (`parse_sentence_mode`, `Op::Ornament`), `src/ast.rs`

#### Summary

Classic SPL (ParseMode::Classic) treats every sentence in a dialogue block as an executable statement. A sentence that does not match any known statement form is a parse error. This is faithful to the 2001 spec but produces aesthetically constrained programs: every sentence must be machine-active.

SPL-F2 mode introduces **ornamental no-ops** — sentences that compile to `Op::Ornament`, which has no effect on interpreter state. This allows authors to include literary connective tissue (stage directions in prose form, thematic asides, rhetorical elaborations) that does not affect program semantics.

#### Rules

Four rules govern which sentences are accepted as ornaments in F2 mode:

| Rule | Condition | Result |
|------|-----------|--------|
| O2 | Sentence begins with a reserved operational head (e.g. `Speak`, `Open`, `You are`, `Let us`, `Remember`, `Recall`, `Am I`) | Parse error — same as Classic |
| O3 | Sentence is a valid SPL statement | Compiled normally — same as Classic |
| O4 | Sentence does not begin with a reserved head but contains an incomplete executable skeleton (e.g. `You are` missing an expression) | Parse error — same as Classic |
| O5 | None of the above apply | Accepted as `Op::Ornament` (no-op) |

Rules O2 and O4 prevent silent truncation of malformed statements: a sentence that looks like it is trying to be an assignment or output but fails to parse completely is still an error. Only sentences that make no attempt at executable structure are accepted as ornaments.

#### Implementation

- `src/ast.rs`: `Op::Ornament` variant added to the `Op` enum.
- `src/parser.rs`: `parse_sentence_mode` checks `ParseMode::F2` at the end of its match chain; `has_reserved_head` and `has_executable_skeleton` implement Rules O2/O4; Rule O5 returns `Ok(Op::Ornament)`.
- `src/interpreter.rs`: `Op::Ornament` is a no-op; the interpreter advances the program counter without modifying state.

#### Backward compatibility

`ParseMode::Classic` is unchanged. All 8 bundled example programs produce identical output in Classic and F2 mode (none contain ornamental sentences). F2 is a strict superset of Classic for programs that only use Classic constructs.

---

### D-004 — SPL-F2 Staging Extensions

**Date:** 2026-04  
**Status:** Extension (opt-in via `--f2` / `ParseMode::F2`; off by default)  
**Affects:** `src/parser.rs` (`parse_speech_header_f2`), `src/ast.rs` (`SpeechMode`, `Op::BeginSpeech`), `src/interpreter.rs`

#### Summary

Classic SPL enforces a strict two-character staging rule: exactly two characters must be on stage for any dialogue, and the second-person pronoun (`you` / `thee`) implicitly refers to whoever is not the speaker. This is a computational constraint that happens to produce plays with exactly two characters on stage at all times — a limitation that has no basis in Shakespearean dramaturgy.

SPL-F2 introduces explicit staging annotations in dialogue headers that relax the two-character rule while preserving deterministic second-person binding.

#### New header forms

| Syntax | Effect |
|--------|--------|
| `Hamlet:` | Classic — unchanged; requires exactly 2 on stage |
| `Hamlet to Ophelia:` | Explicit addressee; any stage count; `you` = Ophelia |
| `Hamlet aside:` | No second-person binding; `you` operations forbidden |
| `Hamlet aside to Ophelia:` | Directed aside; computationally binds Ophelia; `you` = Ophelia |
| `Hamlet solus:` | Soliloquy; requires Hamlet alone on stage; `you` operations forbidden |

#### Implementation

- `src/ast.rs`: `SpeechMode` enum (`Normal`, `To(String)`, `Aside`, `AsideTo(String)`, `Solus`) and `Op::BeginSpeech(SpeechMode)` added.
- `src/parser.rs`: `parse_speech_header_f2` parses the extended header syntax. In F2/F1 mode, each dialogue header emits `Op::BeginSpeech(mode)` before any dialogue sentences. In Classic mode, no `BeginSpeech` ops are emitted.
- `src/interpreter.rs`: `Op::BeginSpeech` validates staging constraints at block entry (solus stage count, addressee on stage) and updates `State::speech_mode`. `State::resolve_addressee` dispatches on `speech_mode` to determine the second-person target.

#### Backward compatibility

The `Normal` speech mode replicates Classic behaviour exactly. All 8 bundled example programs are two-character plays and produce identical output in Classic and F2 mode. The two-character constraint remains the default; the new forms are additive.

---

## Changelog

| Version | Date | Entry |
|---------|------|-------|
| 0.1.0 | 2026-04 | D-001: Documented numeric output concatenation ambiguity; no fix (spec-faithful) |
| 0.1.0 | 2026-04 | D-002: Vocabulary provenance — spl2c over zmbc; three-category model; `road` bug fixed |
| 0.1.0 | 2026-04 | D-003: SPL-F2 ornamental sentences (Op::Ornament, Rules O2–O5) |
| 0.1.0 | 2026-04 | D-004: SPL-F2 staging extensions (SpeechMode, BeginSpeech, `to`/`aside`/`solus`) |
