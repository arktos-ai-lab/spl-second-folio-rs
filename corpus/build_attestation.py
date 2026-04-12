"""
build_attestation.py
────────────────────
Downloads Project Gutenberg Complete Works of Shakespeare (EBook #100),
tokenizes all text, and cross-checks every SPL vocabulary word from
spl-rs/src/vocab.rs to produce an attestation report.

Output
  corpus/guten-complete-2026.txt   — raw downloaded text (cached)
  corpus/attested.json             — per-word attestation record
  corpus/attestation_report.txt    — human-readable summary

Attestation criterion: word (lemma-normalized) appears anywhere in
Shakespeare's complete works (plays + sonnets + narrative poems).
Stage directions count. All text counts.

Pack: guten-complete-2026
"""

import re
import json
import pathlib
import urllib.request
import sys

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

HERE = pathlib.Path(__file__).parent          # spl-rs/corpus/
VOCAB_RS = HERE.parent / "src" / "vocab.rs"  # spl-rs/src/vocab.rs
RAW_TEXT = HERE / "guten-complete-2026.txt"
OUT_JSON = HERE / "attested.json"
OUT_REPORT = HERE / "attestation_report.txt"

GUTENBERG_URL = "https://www.gutenberg.org/cache/epub/100/pg100.txt"

# ---------------------------------------------------------------------------
# Download corpus
# ---------------------------------------------------------------------------

def download_corpus() -> str:
    if RAW_TEXT.exists():
        print(f"[corpus] using cached {RAW_TEXT.name} ({RAW_TEXT.stat().st_size:,} bytes)")
        return RAW_TEXT.read_text(encoding="utf-8", errors="replace")
    print(f"[corpus] downloading from Project Gutenberg …")
    with urllib.request.urlopen(GUTENBERG_URL, timeout=60) as resp:
        data = resp.read()
    RAW_TEXT.write_bytes(data)
    text = data.decode("utf-8", errors="replace")
    print(f"[corpus] saved {RAW_TEXT.name} ({len(data):,} bytes)")
    return text

# ---------------------------------------------------------------------------
# Tokenize corpus → attested lemma set
# ---------------------------------------------------------------------------

def build_token_set(text: str) -> set[str]:
    """
    Lowercase, strip punctuation, split on whitespace.
    Also add possessive-stripped forms: "summer's" → "summer".
    Hyphenated compounds kept whole AND split: "fat-kidneyed" → {"fat-kidneyed", "fat", "kidneyed"}.
    """
    tokens: set[str] = set()
    # remove Project Gutenberg header/footer boilerplate
    # PG texts wrap content between *** START ... *** and *** END ... ***
    m = re.search(r"\*{3}\s*START[^\n]*\n", text, re.IGNORECASE)
    if m:
        text = text[m.end():]
    m = re.search(r"\*{3}\s*END[^\n]*", text, re.IGNORECASE)
    if m:
        text = text[:m.start()]

    for raw in text.split():
        word = raw.lower()
        # normalize curly/unicode apostrophes and quotation marks to ASCII apostrophe
        word = word.replace("\u2019", "'").replace("\u2018", "'").replace("\u02bc", "'")
        # strip leading/trailing punctuation (but keep internal hyphens and apostrophes)
        word = re.sub(r"^[^a-z'-]+|[^a-z'-]+$", "", word)
        if not word:
            continue
        tokens.add(word)
        # possessive: strip trailing 's or '
        if word.endswith("'s"):
            tokens.add(word[:-2])
        elif word.endswith("'"):
            tokens.add(word[:-1])
        # hyphenated compound: also add whole and parts
        if "-" in word:
            for part in word.split("-"):
                if part:
                    tokens.add(part)
    return tokens

# ---------------------------------------------------------------------------
# Parse vocab.rs → SPL word lists
# ---------------------------------------------------------------------------

def parse_vocab_rs(path: pathlib.Path) -> dict[str, list[str]]:
    """
    Extracts word lists from vocab.rs by scanning for pub const X: &[&str] = &[...];
    Returns dict mapping constant name → list of words.
    """
    text = path.read_text(encoding="utf-8")
    result: dict[str, list[str]] = {}
    # Match:  pub const NAME: &[&str] = &[ ... ];
    pattern = re.compile(
        r'pub const (\w+):\s*&\[&str\]\s*=\s*&\[(.*?)\];',
        re.DOTALL
    )
    for m in pattern.finditer(text):
        name = m.group(1)
        body = m.group(2)
        # Extract all string literals
        words = re.findall(r'"([^"]+)"', body)
        result[name] = words
    return result

# ---------------------------------------------------------------------------
# Attestation check
# ---------------------------------------------------------------------------

VOCAB_CATEGORIES = [
    ("POSITIVE_ADJECTIVES", "positive-adjective"),
    ("NEUTRAL_ADJECTIVES",  "neutral-adjective"),
    ("NEGATIVE_ADJECTIVES", "negative-adjective"),
    ("POSITIVE_NOUNS",      "positive-noun"),
    ("NEUTRAL_NOUNS",       "neutral-noun"),
    ("NEGATIVE_NOUNS",      "negative-noun"),
]

def attest_word(word: str, tokens: set[str]) -> str:
    """
    Returns 'attested' or 'legacy-only'.
    Handles multi-word entries (e.g. 'stone wall') by requiring ALL tokens present.
    Handles hyphenated entries by checking the compound first, then parts.
    """
    parts = word.lower().split()
    if len(parts) > 1:
        # multi-word: all parts must be attested (compound may not appear verbatim)
        return "attested" if all(p in tokens for p in parts) else "legacy-only"
    # single word (possibly hyphenated)
    return "attested" if word.lower() in tokens else "legacy-only"

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("[vocab] parsing vocab.rs …")
    vocab_lists = parse_vocab_rs(VOCAB_RS)
    for cat, _ in VOCAB_CATEGORIES:
        if cat not in vocab_lists:
            print(f"  WARNING: {cat} not found in vocab.rs", file=sys.stderr)

    print("[corpus] loading Shakespeare …")
    text = download_corpus()
    print("[corpus] tokenizing …")
    tokens = build_token_set(text)
    print(f"[corpus] {len(tokens):,} unique tokens")

    # Build attestation records
    records: list[dict] = []
    for const_name, spl_class in VOCAB_CATEGORIES:
        words = vocab_lists.get(const_name, [])
        for word in words:
            status = attest_word(word, tokens)
            records.append({
                "word": word,
                "spl_class": spl_class,
                "const": const_name,
                "pack": "guten-complete-2026",
                "status": status,
            })

    # Save JSON
    OUT_JSON.write_text(json.dumps(records, indent=2), encoding="utf-8")
    print(f"[output] wrote {OUT_JSON.name}")

    # Build report
    attested     = [r for r in records if r["status"] == "attested"]
    legacy_only  = [r for r in records if r["status"] == "legacy-only"]

    lines = []
    lines.append("SPL Vocabulary Attestation Report")
    lines.append(f"Pack: guten-complete-2026")
    lines.append(f"Source: Project Gutenberg Complete Works of Shakespeare (EBook #100)")
    lines.append(f"Criterion: lemma present anywhere in Shakespeare's complete works")
    lines.append("")
    lines.append(f"Total words checked : {len(records)}")
    lines.append(f"Attested            : {len(attested)}  ({100*len(attested)/len(records):.1f}%)")
    lines.append(f"Legacy-only         : {len(legacy_only)}  ({100*len(legacy_only)/len(records):.1f}%)")
    lines.append("")

    for const_name, spl_class in VOCAB_CATEGORIES:
        cat_words = [r for r in records if r["const"] == const_name]
        cat_legacy = [r for r in cat_words if r["status"] == "legacy-only"]
        lines.append(f"── {spl_class} ({len(cat_words)} words, {len(cat_legacy)} legacy-only) ──")
        if cat_legacy:
            for r in cat_legacy:
                lines.append(f"  LEGACY-ONLY  {r['word']}")
        else:
            lines.append("  (all attested)")
        lines.append("")

    report = "\n".join(lines)
    OUT_REPORT.write_text(report, encoding="utf-8")
    print(f"[output] wrote {OUT_REPORT.name}")
    print()
    print(report)

if __name__ == "__main__":
    main()
