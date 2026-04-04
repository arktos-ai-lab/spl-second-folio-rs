# Design Decisions and Departures from the Original SPL Specification

This document records every deliberate departure from the original Shakespeare Programming Language specification (Hasselström & Åslund, KTH 2001) made in `shakespeare-spl-folio-rs`, along with the rationale for each decision. It is intended as a permanent reference for contributors, researchers, and users who need to understand where and why this implementation diverges from the canonical source.

For behaviour that is identical to the original, no entry is made here. For ambiguities resolved by reference to the `zmbc/shakespearelang` Python implementation, see §12 of `SPECIFICATION.md`.

---

## Background

The original SPL specification exists as an informal document bundled with the `spl2c` compiler (sourceforge, 2001). It was never peer-reviewed or formally published. Two decades of independent re-implementation have produced a small family of interpreters that diverge silently on edge cases. `shakespeare-spl-folio-rs` aims to be the reference implementation: fully specified, tested, and documented. Where it departs from the 2001 original, it does so deliberately and records the departure here.

---

## Decisions

---

### D-001 — Character Name Vocabulary Expansion

**Date:** 2026-04  
**Status:** Accepted  
**Affects:** `src/vocab.rs`, character declaration parsing, `[Enter …]` / `[Exit …]` parsing

#### Summary

The original `spl2c` distribution recognised approximately 152 character names, drawn informally from a subset of Shakespeare's plays. `shakespeare-spl-folio-rs` expands this to 400+ names, sourced from the full Open Source Shakespeare character database (opensourceshakespeare.org).

#### Original behaviour

The 2001 `spl2c` release bundled a fixed list of roughly 152 names. The list was not exhaustive, not documented as a normative constraint, and contained no filtering criteria. It was simply the set of names the authors happened to include.

#### Departure

`vocab.rs` contains every individually named character appearing in the Open Source Shakespeare database (1,223 entries), filtered by the following rules:

| Rule | Decision |
|------|----------|
| Characters with a proper name (Aaron, Hamlet, Friar Laurence …) | **Included** |
| Generic roles without a name (Messenger, Servant, Captain …) | **Excluded** |
| Numbered / ordinal characters (First Citizen, Second Murderer …) | **Excluded** |
| Collective nouns (All, Both, Citizens, Soldiers …) | **Excluded** |
| Title-only roles with no proper name (Lord Mayor, Ambassador …) | **Excluded** |

This yields 400+ accepted names. All names from the original 152-name list are a strict subset of this expanded list, so all programs valid under `spl2c` remain valid here.

#### Rationale

1. **Completeness.** The original list was an artefact of manual curation, not a principled boundary. There is no semantic reason why `Aaron` (present in spl2c) should be valid but `Adriana` (absent) should not.
2. **Reproducibility.** The filtering rules above are explicit and reproducible from a publicly available database, unlike the original undocumented list.
3. **Research use.** The broader vocabulary meaningfully increases the space of syntactically valid programs that can be synthesised for training data purposes.
4. **Backward compatibility.** The original ~152 names remain valid. No existing program is broken.

#### Compatibility implications

Programs using names from the extended vocabulary (e.g. `Diana`) will not run under `spl2c` or `zmbc/shakespearelang`. This creates a three-tier compatibility model:

| Tier | Vocabulary | Compatible with |
|------|------------|-----------------|
| spl2c-compat | ~152 original names | spl2c, zmbc, spl-rs |
| zmbc-compat | ~152 names + zmbc additions | zmbc, spl-rs |
| spl-rs extended | 400+ names | spl-rs only |

A future `--strict` flag may enforce the original 152-name list for maximum compatibility. See the roadmap in `README.md`.

#### References

- Hasselström & Åslund (2001), spl2c v1.2.1, http://shakespeare.sourceforge.net
- Open Source Shakespeare, https://opensourceshakespeare.org
- `SPECIFICATION.md` §12.4 — Character name vocabulary scope

---

## Changelog

| Version | Date | Entry |
|---------|------|-------|
| 0.1.0 | 2026-04 | D-001: Character name vocabulary expanded from ~152 to 400+ names |
