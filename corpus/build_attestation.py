"""
build_attestation.py
────────────────────
Downloads three corpus packs and cross-checks every SPL vocabulary word
and character name from spl-rs/src/vocab.rs.

Packs
  guten-complete-2026   Project Gutenberg Complete Works (EBook #100)  ~5.6 MB
  folger-complete-2026  Folger Digital Texts complete XML              ~20 MB
  eebo-period-2026      EEBO-TCP Phase 1, 1575-1660 (individual files)
                        skipped unless --eebo is passed
                        default sample: 1000 docs; --eebo-full = all 12k

Output (split by category)
  corpus/attested_vocab.json    vocabulary words (nouns, adjectives)
  corpus/attested_names.json    character names (personae)
  corpus/attestation_report.txt human-readable summary

Attestation rules
  Vocabulary : lemma present in ANY pack's token set
  Names      : full name phrase found in ANY pack's lowercased text
               known typos get attested-alias status

Usage
  python build_attestation.py              # guten + folger only
  python build_attestation.py --eebo       # + EEBO sample (1000 docs)
  python build_attestation.py --eebo-full  # + full EEBO 1575-1660 (~12k docs)
"""

import re
import csv
import json
import sys
import zipfile
import io
import pathlib
import random
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

HERE     = pathlib.Path(__file__).parent          # spl-rs/corpus/
VOCAB_RS = HERE.parent / "src" / "vocab.rs"       # spl-rs/src/vocab.rs

OUT_VOCAB  = HERE / "attested_vocab.json"
OUT_NAMES  = HERE / "attested_names.json"
OUT_REPORT = HERE / "attestation_report.txt"

# ---------------------------------------------------------------------------
# Known typos in SPL 1.2.1 spec, preserved for backward compat
# ---------------------------------------------------------------------------

NAME_TYPOS = {
    "cymberline": "cymbeline",
    "hermonie":   "hermione",
}

# ---------------------------------------------------------------------------
# Download helpers
# ---------------------------------------------------------------------------

def download_bytes(url: str, timeout: int = 120) -> bytes:
    with urllib.request.urlopen(url, timeout=timeout) as resp:
        return resp.read()

def download_file(url: str, dest: pathlib.Path) -> None:
    print(f"  downloading {dest.name} ...")
    data = download_bytes(url)
    dest.write_bytes(data)
    print(f"  saved {dest.name} ({len(data):,} bytes)")

# ---------------------------------------------------------------------------
# Text extraction helpers
# ---------------------------------------------------------------------------

def strip_pg_boilerplate(text: str) -> str:
    m = re.search(r"\*{3}\s*START[^\n]*\n", text, re.IGNORECASE)
    if m:
        text = text[m.end():]
    m = re.search(r"\*{3}\s*END[^\n]*", text, re.IGNORECASE)
    if m:
        text = text[:m.start()]
    return text

def strip_xml_tags(text: str) -> str:
    return re.sub(r"<[^>]+>", " ", text)

def normalize(text: str) -> str:
    text = text.lower()
    text = text.replace("\u2019", "'").replace("\u2018", "'").replace("\u02bc", "'")
    return text

# ---------------------------------------------------------------------------
# Token set builder
# ---------------------------------------------------------------------------

def build_token_set(text: str) -> set[str]:
    tokens: set[str] = set()
    for raw in text.split():
        word = re.sub(r"^[^a-z'-]+|[^a-z'-]+$", "", raw)
        if not word:
            continue
        tokens.add(word)
        if word.endswith("'s"):
            tokens.add(word[:-2])
        elif word.endswith("'"):
            tokens.add(word[:-1])
        if "-" in word:
            for part in word.split("-"):
                if part:
                    tokens.add(part)
    return tokens

# ---------------------------------------------------------------------------
# Pack loaders
# ---------------------------------------------------------------------------

GUTEN_URL  = "https://www.gutenberg.org/cache/epub/100/pg100.txt"
FOLGER_URL = "https://www.folgerdigitaltexts.org/download/xml/FolgerDigitalTexts_XML_Complete.zip"
EEBO_META_URL = "https://github.com/textcreationpartnership/Texts/archive/refs/heads/master.zip"
EEBO_RAW_URL  = "https://raw.githubusercontent.com/textcreationpartnership/{id}/master/{id}.xml"

EEBO_DATE_MIN = 1575
EEBO_DATE_MAX = 1660
EEBO_DEFAULT_SAMPLE = 1000
EEBO_WORKERS = 10


def load_guten() -> tuple[set[str], str]:
    cache = HERE / "guten-complete-2026.txt"
    if not cache.exists():
        print("[guten] downloading ...")
        download_file(GUTEN_URL, cache)
    else:
        print(f"[guten] using cache ({cache.stat().st_size:,} bytes)")
    raw = cache.read_text(encoding="utf-8", errors="replace")
    text = normalize(strip_pg_boilerplate(raw))
    tokens = build_token_set(text)
    print(f"[guten] {len(tokens):,} unique tokens")
    return tokens, text


def load_folger() -> tuple[set[str], str]:
    cache = HERE / "folger-complete-2026.zip"
    if not cache.exists():
        print("[folger] downloading ...")
        download_file(FOLGER_URL, cache)
    else:
        print(f"[folger] using cache ({cache.stat().st_size:,} bytes)")
    parts = []
    with zipfile.ZipFile(cache) as zf:
        for name in zf.namelist():
            if not name.lower().endswith((".xml", ".txt")):
                continue
            try:
                raw = zf.read(name).decode("utf-8", errors="replace")
            except Exception:
                continue
            parts.append(strip_xml_tags(raw))
    text = normalize("\n".join(parts))
    tokens = build_token_set(text)
    print(f"[folger] {len(tokens):,} unique tokens")
    return tokens, text


def _eebo_doc_ids(meta_zip: pathlib.Path, full: bool) -> list[str]:
    """Parse TCP.csv and return filtered document IDs."""
    with zipfile.ZipFile(meta_zip) as zf:
        raw = zf.read("Texts-master/TCP.csv").decode("utf-8", errors="replace")
    reader = csv.DictReader(io.StringIO(raw))
    ids = []
    for row in reader:
        if row.get("Status", "").strip() != "Free":
            continue
        try:
            year = int(str(row.get("Date", "0")).strip()[:4])
        except ValueError:
            continue
        if EEBO_DATE_MIN <= year <= EEBO_DATE_MAX:
            ids.append(row["TCP"].strip())
    if not full and len(ids) > EEBO_DEFAULT_SAMPLE:
        random.seed(42)                         # reproducible sample
        ids = random.sample(ids, EEBO_DEFAULT_SAMPLE)
    return ids


def _fetch_eebo_doc(doc_id: str) -> str:
    url = EEBO_RAW_URL.format(id=doc_id)
    try:
        data = download_bytes(url, timeout=30)
        text = data.decode("utf-8", errors="replace")
        return strip_xml_tags(text)
    except Exception:
        return ""


def load_eebo(full: bool = False) -> tuple[set[str], str]:
    meta_cache = HERE / "eebo-period-2026.zip"
    if not meta_cache.exists():
        print("[eebo] downloading metadata index ...")
        download_file(EEBO_META_URL, meta_cache)
    else:
        print(f"[eebo] using cached metadata ({meta_cache.stat().st_size:,} bytes)")

    ids = _eebo_doc_ids(meta_cache, full)
    label = "full" if full else f"sample n={len(ids)}"
    print(f"[eebo] fetching {len(ids)} documents ({label}, {EEBO_DATE_MIN}-{EEBO_DATE_MAX}) ...")

    parts: list[str] = []
    done = 0
    with ThreadPoolExecutor(max_workers=EEBO_WORKERS) as pool:
        futures = {pool.submit(_fetch_eebo_doc, doc_id): doc_id for doc_id in ids}
        for fut in as_completed(futures):
            text = fut.result()
            if text:
                parts.append(text)
            done += 1
            if done % 100 == 0:
                print(f"  [eebo] {done}/{len(ids)} ...")

    full_text = normalize("\n".join(parts))
    tokens = build_token_set(full_text)
    print(f"[eebo] {len(tokens):,} unique tokens from {len(parts)} documents")
    return tokens, full_text

# ---------------------------------------------------------------------------
# Parse vocab.rs
# ---------------------------------------------------------------------------

def parse_vocab_rs(path: pathlib.Path) -> dict[str, list[str]]:
    text = path.read_text(encoding="utf-8")
    result: dict[str, list[str]] = {}
    pattern = re.compile(
        r'pub const (\w+):\s*&\[&str\]\s*=\s*&\[(.*?)\];',
        re.DOTALL
    )
    for m in pattern.finditer(text):
        result[m.group(1)] = re.findall(r'"([^"]+)"', m.group(2))
    return result

# ---------------------------------------------------------------------------
# Attestation
# ---------------------------------------------------------------------------

VOCAB_CATEGORIES = [
    ("POSITIVE_ADJECTIVES", "positive-adjective"),
    ("NEUTRAL_ADJECTIVES",  "neutral-adjective"),
    ("NEGATIVE_ADJECTIVES", "negative-adjective"),
    ("POSITIVE_NOUNS",      "positive-noun"),
    ("NEUTRAL_NOUNS",       "neutral-noun"),
    ("NEGATIVE_NOUNS",      "negative-noun"),
]


def attest_word(word: str, token_sets: dict[str, set[str]]) -> tuple[str, list[str]]:
    found_in = []
    parts = word.lower().split()
    for pack_name, tokens in token_sets.items():
        if len(parts) > 1:
            if all(p in tokens for p in parts):
                found_in.append(pack_name)
        else:
            if word.lower() in tokens:
                found_in.append(pack_name)
    return ("attested" if found_in else "legacy-only", found_in)


def attest_name(name: str, corpus_texts: dict[str, str]) -> tuple[str, str | None, list[str]]:
    is_typo = name in NAME_TYPOS
    lookup  = NAME_TYPOS[name] if is_typo else name
    pattern = re.compile(r'\b' + re.escape(lookup) + r'\b')
    found_in = []
    for pack_name, text in corpus_texts.items():
        if pattern.search(text):
            found_in.append(pack_name)
    if found_in:
        return ("attested-alias" if is_typo else "attested"), lookup, found_in
    return "legacy-only", None, []

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    run_eebo      = "--eebo" in sys.argv or "--eebo-full" in sys.argv
    run_eebo_full = "--eebo-full" in sys.argv

    print("[vocab] parsing vocab.rs ...")
    vocab_lists = parse_vocab_rs(VOCAB_RS)

    token_sets:   dict[str, set[str]] = {}
    corpus_texts: dict[str, str]      = {}

    t, tx = load_guten();  token_sets["guten-complete-2026"]  = t; corpus_texts["guten-complete-2026"]  = tx
    t, tx = load_folger(); token_sets["folger-complete-2026"] = t; corpus_texts["folger-complete-2026"] = tx
    if run_eebo:
        t, tx = load_eebo(full=run_eebo_full)
        token_sets["eebo-period-2026"]  = t
        corpus_texts["eebo-period-2026"] = tx

    active_packs = list(token_sets.keys())
    print(f"[packs] active: {active_packs}")

    # Vocabulary
    vocab_records: list[dict] = []
    for const_name, spl_class in VOCAB_CATEGORIES:
        for word in vocab_lists.get(const_name, []):
            status, found_in = attest_word(word, token_sets)
            vocab_records.append({
                "word":        word,
                "spl_class":   spl_class,
                "const":       const_name,
                "status":      status,
                "attested_in": found_in,
                "attested_as": None,
            })

    # Personae
    name_records: list[dict] = []
    for name in vocab_lists.get("CHARACTER_NAMES", []):
        status, attested_as, found_in = attest_name(name, corpus_texts)
        name_records.append({
            "word":        name,
            "spl_class":   "character-name",
            "const":       "CHARACTER_NAMES",
            "status":      status,
            "attested_in": found_in,
            "attested_as": attested_as,
        })

    OUT_VOCAB.write_text(json.dumps(vocab_records, indent=2), encoding="utf-8")
    OUT_NAMES.write_text(json.dumps(name_records,  indent=2), encoding="utf-8")
    print(f"[output] wrote {OUT_VOCAB.name}  ({len(vocab_records)} entries)")
    print(f"[output] wrote {OUT_NAMES.name}  ({len(name_records)} entries)")

    # Report
    def pct(n, total): return f"{100*n/total:.1f}%" if total else "n/a"
    def count(recs, s): return sum(1 for r in recs if r["status"] == s)

    v_att = count(vocab_records, "attested");     v_leg = count(vocab_records, "legacy-only")
    n_att = count(name_records,  "attested");     n_ali = count(name_records, "attested-alias")
    n_leg = count(name_records,  "legacy-only")
    total = len(vocab_records) + len(name_records)

    lines = []
    lines.append("SPL Vocabulary & Personae Attestation Report")
    lines.append(f"Active packs : {', '.join(active_packs)}")
    lines.append(f"Scope        : plays + sonnets + narrative poems; stage directions included")
    lines.append("")
    lines.append(f"VOCABULARY  {len(vocab_records):>4} words   attested {v_att} ({pct(v_att,len(vocab_records))})   legacy-only {v_leg} ({pct(v_leg,len(vocab_records))})")
    lines.append(f"PERSONAE    {len(name_records):>4} names   attested {n_att} ({pct(n_att,len(name_records))})   alias {n_ali}   legacy-only {n_leg} ({pct(n_leg,len(name_records))})")
    lines.append(f"TOTAL       {total:>4}")
    lines.append("")

    for const_name, spl_class in VOCAB_CATEGORIES:
        cat = [r for r in vocab_records if r["const"] == const_name]
        leg = [r for r in cat if r["status"] == "legacy-only"]
        lines.append(f"-- {spl_class} ({len(cat)} words, {len(leg)} legacy-only) --")
        for r in leg:
            lines.append(f"  LEGACY-ONLY  {r['word']:<25}  attested_in: {r['attested_in']}")
        if not leg:
            lines.append("  (all attested)")
        lines.append("")

    lines.append(f"-- character-name ({len(name_records)} names) --")
    for r in name_records:
        packs_str = ", ".join(r["attested_in"]) if r["attested_in"] else "none"
        if r["status"] == "attested":
            lines.append(f"  attested        {r['word']:<35}  packs: {packs_str}")
        elif r["status"] == "attested-alias":
            lines.append(f"  attested-alias  {r['word']:<35}  -> {r['attested_as']}  packs: {packs_str}")
        else:
            lines.append(f"  LEGACY-ONLY     {r['word']}")
    lines.append("")

    report = "\n".join(lines)
    OUT_REPORT.write_text(report, encoding="utf-8")
    print(f"[output] wrote {OUT_REPORT.name}")

if __name__ == "__main__":
    main()
