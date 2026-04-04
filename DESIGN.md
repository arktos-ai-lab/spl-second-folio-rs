# Design Decisions and Departures from the Original SPL Specification

This document records every deliberate departure from the original Shakespeare Programming Language specification (Hasselström & Åslund, KTH 2001) made in `shakespeare-spl-folio-rs`, along with the rationale for each decision. It is intended as a permanent reference for contributors, researchers, and users who need to understand where and why this implementation diverges from the canonical source.

For behaviour that is identical to the original, no entry is made here. For ambiguities resolved by reference to the `zmbc/shakespearelang` Python implementation, see §12 of `SPECIFICATION.md`.

---

## Background

The original SPL specification exists as an informal document bundled with the `spl2c` compiler (sourceforge, 2001). It was never peer-reviewed or formally published. Two decades of independent re-implementation have produced a small family of interpreters that diverge silently on edge cases. `shakespeare-spl-folio-rs` aims to be the reference implementation: fully specified, tested, and documented. Where it departs from the 2001 original, it does so deliberately and records the departure here.

---

## Decisions

---

### D-001 — Character Name Vocabulary

**Date:** 2026-04 (revised 2026-04)  
**Status:** Accepted  
**Affects:** `src/vocab.rs`, character declaration parsing, `[Enter …]` / `[Exit …]` parsing

#### Summary

`shakespeare-spl-folio-rs` uses the canonical 152-name character list from the original `spl2c` distribution (Hasselström & Åslund, 2001), cross-checked against the `zmbc/shakespearelang` EBNF grammar and the `drsam94/Spl` character wordlist.

An earlier version of this codebase expanded the list to 400+ names using the Open Source Shakespeare database (opensourceshakespeare.org). That expansion has been reverted: the OSS database is a structured schema built from editorial metadata, not from the primary Shakespearean text corpus, and is therefore not an appropriate provenance source for SPL vocabulary (see SPL 1.1 design notes).

#### Original behaviour

The 2001 `spl2c` release bundled a fixed list of 152 names drawn from a selection of Shakespeare's plays. Two typos in the original are preserved as-is for compatibility:

| In spec | Should be |
|---------|-----------|
| `Cymberline` | Cymbeline |
| `Hermonie` | Hermione |

#### Current behaviour

`vocab.rs` contains exactly the canonical 152 names. Any program using a name outside this list will be rejected at parse time (character declaration stage). Because character references in expressions are already scoped to names *declared in the current play*, the size of the global allow-list matters only for the declaration check.

#### Compatibility

All programs valid under `spl2c` or `zmbc/shakespearelang` remain valid here. Programs that relied on the previous OSS-expanded list (names such as `Aaron`, `Ariel`, `Caliban`, etc.) must be updated to use canonical names or declare characters using canonical alternatives.

#### References

- Hasselström & Åslund (2001), spl2c v1.2.1, http://shakespeare.sourceforge.net
- zmbc/shakespearelang EBNF — `shakespeare.ebnf`
- drsam94/Spl — `include/character.wordlist`

---

## Changelog

| Version | Date | Entry |
|---------|------|-------|
| 0.1.0 | 2026-04 | D-001: Character name vocabulary initially expanded from ~152 to 400+ OSS names |
| 0.1.1 | 2026-04 | D-001: Reverted to canonical 152-name spl2c list; OSS expansion removed |
