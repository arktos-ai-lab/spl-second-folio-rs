"""
build_attestation.py
────────────────────
Downloads three corpus packs and cross-checks every SPL vocabulary word
and character name from spl-rs/src/vocab.rs.

Canonical pack  oss-moby-2026
  guten-complete-2026   Project Gutenberg Complete Works (EBook #100)  ~5.6 MB
  folger-complete-2026  Folger Digital Texts complete XML              ~20 MB

Supplementary pack (opt-in)
  eebo-period-2026      EEBO-TCP Phase 1, 1575-1660 (individual files)
                        --eebo: sample of 1000 docs
                        --eebo-full: all ~12k docs

Output
  corpus/attested_vocab.json    vocabulary words (nouns, adjectives)
  corpus/attested_names.json    character names (personae)
  corpus/candidates_vocab.json  §6.4 candidate words for vocabulary expansion
  corpus/attestation_report.txt human-readable summary

Attestation rules
  Vocabulary : exact token OR WordNet lemma present in ANY pack's token set
               lemma variants tried: WordNet lemmatize with pos a/s/v/n
  Names      : full name phrase found in ANY pack's lowercased text
               known typos get attested-alias status

New fields (v2)
  frequency  : total occurrences of the attested form in guten+folger combined
  plays      : Folger play codes where the word/name appears
  attested_as: lemma or corrected-typo form used to find the word (None = exact match)

Usage
  python build_attestation.py              # guten + folger only (canonical)
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
import unicodedata
import urllib.request
from collections import Counter
from concurrent.futures import ThreadPoolExecutor, as_completed

import nltk
nltk.download("wordnet",      quiet=True)
nltk.download("sentiwordnet", quiet=True)
from nltk.stem import WordNetLemmatizer
from nltk.corpus import wordnet as wn
from nltk.corpus import sentiwordnet as swn

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

HERE     = pathlib.Path(__file__).parent          # spl-rs/corpus/
VOCAB_RS = HERE.parent / "src" / "vocab.rs"       # spl-rs/src/vocab.rs

OUT_VOCAB       = HERE / "attested_vocab.json"
OUT_NAMES       = HERE / "attested_names.json"
OUT_CANDIDATES  = HERE / "candidates_vocab.json"
OUT_REPORT      = HERE / "attestation_report.txt"

# Minimum corpus frequency for candidate extraction (§6.4)
CANDIDATE_MIN_FREQ = 10
CANDIDATES_PER_CLASS = 60

_lemmatizer = WordNetLemmatizer()

# ---------------------------------------------------------------------------
# Known typos in SPL 1.2.1 spec, preserved for backward compat
# ---------------------------------------------------------------------------

NAME_TYPOS = {
    "cymberline":        "cymbeline",
    "hermonie":          "hermione",
    "friar laurence":    "friar lawrence",
    "domitius enobarus": "enobarbus",
    "tintinius":         "titinius",
    "queen elinor":      "eleanor",
    "launcelot gobbo":   "lancelet gobbo",
    "salanio":           "solanio",
    "lymoges":           "limoges",
    "king lewis xi":     "lewis",
    "grandpre":          "grandpre",
}

# ---------------------------------------------------------------------------
# Folger play-code → full title
# ---------------------------------------------------------------------------

FOLGER_PLAY_NAMES: dict[str, str] = {
    "1H4": "Henry IV Part 1",
    "1H6": "Henry VI Part 1",
    "2H4": "Henry IV Part 2",
    "2H6": "Henry VI Part 2",
    "3H6": "Henry VI Part 3",
    "AWW": "All's Well That Ends Well",
    "AYL": "As You Like It",
    "Ado": "Much Ado About Nothing",
    "Ant": "Antony and Cleopatra",
    "Cor": "Coriolanus",
    "Cym": "Cymbeline",
    "Err": "The Comedy of Errors",
    "H5":  "Henry V",
    "H8":  "Henry VIII",
    "Ham": "Hamlet",
    "JC":  "Julius Caesar",
    "Jn":  "King John",
    "LLL": "Love's Labour's Lost",
    "Lr":  "King Lear",
    "Luc": "The Rape of Lucrece",
    "MM":  "Measure for Measure",
    "MND": "A Midsummer Night's Dream",
    "MV":  "The Merchant of Venice",
    "Mac": "Macbeth",
    "Oth": "Othello",
    "Per": "Pericles",
    "PhT": "The Phoenix and the Turtle",
    "R2":  "Richard II",
    "R3":  "Richard III",
    "Rom": "Romeo and Juliet",
    "Shr": "The Taming of the Shrew",
    "Son": "Sonnets",
    "TGV": "The Two Gentlemen of Verona",
    "TN":  "Twelfth Night",
    "TNK": "The Two Noble Kinsmen",
    "Tim": "Timon of Athens",
    "Tit": "Titus Andronicus",
    "Tmp": "The Tempest",
    "Tro": "Troilus and Cressida",
    "Ven": "Venus and Adonis",
    "WT":  "The Winter's Tale",
    "Wiv": "The Merry Wives of Windsor",
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
# Text helpers
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
    text = unicodedata.normalize("NFD", text)
    text = "".join(c for c in text if unicodedata.category(c) != "Mn")
    return text

# ---------------------------------------------------------------------------
# Token data builder  (set + Counter in one pass)
# ---------------------------------------------------------------------------

def build_token_data(text: str) -> tuple[set[str], Counter]:
    tokens: set[str] = set()
    counts: Counter  = Counter()
    for raw in text.split():
        word = re.sub(r"^[^a-z'-]+|[^a-z'-]+$", "", raw)
        if not word:
            continue
        tokens.add(word)
        counts[word] += 1
        if word.endswith("'s"):
            base = word[:-2]
            tokens.add(base)
            counts[base] += 1
        elif word.endswith("'"):
            base = word[:-1]
            tokens.add(base)
            counts[base] += 1
        if "-" in word:
            for part in word.split("-"):
                if part:
                    tokens.add(part)
                    counts[part] += 1
    return tokens, counts

def build_token_set(text: str) -> set[str]:
    return build_token_data(text)[0]

# ---------------------------------------------------------------------------
# Pack loaders
# ---------------------------------------------------------------------------

GUTEN_URL     = "https://www.gutenberg.org/cache/epub/100/pg100.txt"
FOLGER_URL    = "https://www.folgerdigitaltexts.org/download/xml/FolgerDigitalTexts_XML_Complete.zip"
EEBO_META_URL = "https://github.com/textcreationpartnership/Texts/archive/refs/heads/master.zip"
EEBO_RAW_URL  = "https://raw.githubusercontent.com/textcreationpartnership/{id}/master/{id}.xml"

EEBO_DATE_MIN      = 1575
EEBO_DATE_MAX      = 1660
EEBO_DEFAULT_SAMPLE = 1000
EEBO_WORKERS       = 10


def load_guten() -> tuple[set[str], str, Counter]:
    cache = HERE / "guten-complete-2026.txt"
    if not cache.exists():
        print("[guten] downloading ...")
        download_file(GUTEN_URL, cache)
    else:
        print(f"[guten] using cache ({cache.stat().st_size:,} bytes)")
    raw  = cache.read_text(encoding="utf-8", errors="replace")
    text = normalize(strip_pg_boilerplate(raw))
    tokens, counts = build_token_data(text)
    print(f"[guten] {len(tokens):,} unique tokens")
    return tokens, text, counts


def load_folger() -> tuple[set[str], str, dict[str, str], Counter]:
    """Return (all_tokens, full_text, play_texts, counts)."""
    cache = HERE / "folger-complete-2026.zip"
    if not cache.exists():
        print("[folger] downloading ...")
        download_file(FOLGER_URL, cache)
    else:
        print(f"[folger] using cache ({cache.stat().st_size:,} bytes)")
    parts: list[str] = []
    play_texts: dict[str, str] = {}
    with zipfile.ZipFile(cache) as zf:
        for name in zf.namelist():
            if not name.lower().endswith(".xml"):
                continue
            base = pathlib.Path(name).stem
            try:
                raw = zf.read(name).decode("utf-8", errors="replace")
            except Exception:
                continue
            play_text = normalize(strip_xml_tags(raw))
            parts.append(play_text)
            if base in FOLGER_PLAY_NAMES:
                play_texts[base] = play_text
    text = "\n".join(parts)
    tokens, counts = build_token_data(text)
    print(f"[folger] {len(tokens):,} unique tokens across {len(play_texts)} plays")
    return tokens, text, play_texts, counts


def _eebo_doc_ids(meta_zip: pathlib.Path, full: bool) -> list[str]:
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
        random.seed(42)
        ids = random.sample(ids, EEBO_DEFAULT_SAMPLE)
    return ids


def _fetch_eebo_doc(doc_id: str) -> str:
    url = EEBO_RAW_URL.format(id=doc_id)
    try:
        data = download_bytes(url, timeout=30)
        return strip_xml_tags(data.decode("utf-8", errors="replace"))
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
# Lemmatization
# ---------------------------------------------------------------------------

def lemma_candidates(word: str, spl_class: str) -> list[str]:
    """Return WordNet lemma variants to try if exact match fails."""
    seen: dict[str, None] = {}  # ordered dedup
    w = word.lower()
    if "adjective" in spl_class:
        for pos in ("a", "s", "v"):   # adj, satellite-adj, verb (-ing forms)
            l = _lemmatizer.lemmatize(w, pos)
            if l != w:
                seen[l] = None
    elif "noun" in spl_class:
        for pos in ("n", "v"):
            l = _lemmatizer.lemmatize(w, pos)
            if l != w:
                seen[l] = None
    return list(seen)

# ---------------------------------------------------------------------------
# Attestation
# ---------------------------------------------------------------------------

VOCAB_CATEGORIES = [
    ("POSITIVE_ADJECTIVES",   "positive-adjective"),
    ("NEUTRAL_ADJECTIVES",    "neutral-adjective"),
    ("NEGATIVE_ADJECTIVES",   "negative-adjective"),
    ("POSITIVE_NOUNS",        "positive-noun"),
    ("NEUTRAL_NOUNS",         "neutral-noun"),
    ("NEGATIVE_NOUNS",        "negative-noun"),
    ("POSITIVE_COMPARATIVES", "positive-comparative"),
    ("NEGATIVE_COMPARATIVES", "negative-comparative"),
]


def _word_freq(parts: list[str], counter: Counter) -> int:
    """Frequency of a (possibly multi-token) word: min across parts."""
    return min((counter.get(p, 0) for p in parts), default=0)


def attest_word(
    word: str,
    spl_class: str,
    token_sets: dict[str, set[str]],
    canonical_counter: Counter,
) -> tuple[str, list[str], str | None, int]:
    """Return (status, found_in_packs, attested_as, frequency).

    Tries exact match first, then WordNet lemma variants.
    frequency counts only canonical packs (guten+folger), not EEBO.
    """
    parts = word.lower().split()

    def _check(lookup_parts: list[str]) -> list[str]:
        found = []
        for pack, tokens in token_sets.items():
            if all(p in tokens for p in lookup_parts):
                found.append(pack)
        return found

    found_in = _check(parts)
    if found_in:
        return "attested", found_in, None, _word_freq(parts, canonical_counter)

    for lemma in lemma_candidates(word, spl_class):
        lemma_parts = lemma.split()
        found_in = _check(lemma_parts)
        if found_in:
            return "attested", found_in, lemma, _word_freq(lemma_parts, canonical_counter)

    return "legacy-only", [], None, 0


def attest_name(
    name: str,
    corpus_texts: dict[str, str],
    canonical_counter: Counter,
) -> tuple[str, str | None, list[str], int]:
    is_typo = name in NAME_TYPOS
    lookup  = NAME_TYPOS[name] if is_typo else name
    pattern = re.compile(r'\b' + re.escape(lookup) + r'\b')
    found_in = [pack for pack, text in corpus_texts.items() if pattern.search(text)]
    if found_in:
        freq = _word_freq(lookup.split(), canonical_counter)
        return ("attested-alias" if is_typo else "attested"), lookup, found_in, freq
    return "legacy-only", None, [], 0

# ---------------------------------------------------------------------------
# Play-level provenance
# ---------------------------------------------------------------------------

def build_play_token_sets(play_texts: dict[str, str]) -> dict[str, set[str]]:
    return {code: build_token_set(text) for code, text in play_texts.items()}


def plays_for_word(word: str, play_token_sets: dict[str, set[str]]) -> list[str]:
    parts = word.lower().split()
    return sorted(
        code for code, tokens in play_token_sets.items()
        if all(p in tokens for p in parts)
    )


def plays_for_name(name: str, play_texts: dict[str, str]) -> list[str]:
    lookup  = NAME_TYPOS.get(name, name)
    pattern = re.compile(r'\b' + re.escape(lookup) + r'\b')
    return sorted(code for code, text in play_texts.items() if pattern.search(text))

# ---------------------------------------------------------------------------
# §6.4 Candidate extraction
# ---------------------------------------------------------------------------

def _swn_scores(lemma: str, pos: str) -> tuple[float, float]:
    """Return (avg_pos_score, avg_neg_score) across SentiWordNet synsets."""
    synsets = list(swn.senti_synsets(lemma, pos))
    if not synsets:
        return 0.0, 0.0
    pos_avg = sum(s.pos_score() for s in synsets) / len(synsets)
    neg_avg = sum(s.neg_score() for s in synsets) / len(synsets)
    return pos_avg, neg_avg


def _classify_candidate(word: str) -> str | None:
    """Return suggested SPL class or None if not classifiable.

    Requires the word to be primarily an adjective or noun in WordNet
    (more adj/noun synsets than verb synsets) to filter out polysemous
    function words that have incidental adjectival senses.
    """
    adj_syns  = wn.synsets(word, pos=wn.ADJ)
    noun_syns = wn.synsets(word, pos=wn.NOUN)
    verb_syns = wn.synsets(word, pos=wn.VERB)

    n_adj  = len(adj_syns)
    n_noun = len(noun_syns)
    n_verb = len(verb_syns)

    # Require primarily adj or noun — not dominated by verb senses
    if n_adj == 0 and n_noun == 0:
        return None
    if n_verb > (n_adj + n_noun):
        return None

    # Prefer adjective classification when ambiguous
    if n_adj >= n_noun:
        pos_sc, neg_sc = _swn_scores(word, "a")
        if pos_sc > 0.2:
            return "positive-adjective"
        if neg_sc > 0.2:
            return "negative-adjective"
        return "neutral-adjective"
    else:
        pos_sc, neg_sc = _swn_scores(word, "n")
        if pos_sc > 0.2:
            return "positive-noun"
        if neg_sc > 0.2:
            return "negative-noun"
        return "neutral-noun"


def extract_candidates(
    canonical_counter: Counter,
    spl_words: set[str],
    play_token_sets: dict[str, set[str]],
) -> list[dict]:
    """Return top attested Shakespeare words not in SPL, classified by role."""
    by_class: dict[str, list[tuple[int, str]]] = {}
    for cls in ["positive-adjective", "neutral-adjective", "negative-adjective",
                "positive-noun", "neutral-noun", "negative-noun"]:
        by_class[cls] = []

    checked = 0
    skipped_freq = 0
    skipped_spl  = 0
    skipped_wn   = 0

    for word, freq in canonical_counter.most_common():
        if freq < CANDIDATE_MIN_FREQ:
            skipped_freq += 1
            continue
        # Skip very-high-frequency words (function words, determiners, etc.)
        if freq > 1500:
            continue
        # Must be a real alphabetic word of reasonable length
        if not re.fullmatch(r"[a-z]{4,}", word):
            continue
        if word in spl_words:
            skipped_spl += 1
            continue
        checked += 1
        cls = _classify_candidate(word)
        if cls is None:
            skipped_wn += 1
            continue
        bucket = by_class[cls]
        if len(bucket) < CANDIDATES_PER_CLASS:
            bucket.append((freq, word))

        if all(len(v) >= CANDIDATES_PER_CLASS for v in by_class.values()):
            break

    print(f"[candidates] checked {checked} tokens  "
          f"skipped freq<{CANDIDATE_MIN_FREQ}: {skipped_freq}  "
          f"in-spl: {skipped_spl}  no-wordnet: {skipped_wn}")

    records: list[dict] = []
    for cls, entries in by_class.items():
        for freq, word in sorted(entries, reverse=True):
            plays = plays_for_word(word, play_token_sets)
            records.append({
                "word":            word,
                "suggested_class": cls,
                "status":          "candidate",
                "frequency":       freq,
                "plays":           plays,
                "n_plays":         len(plays),
            })
    return records

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    run_eebo      = "--eebo" in sys.argv or "--eebo-full" in sys.argv
    run_eebo_full = "--eebo-full" in sys.argv

    print("[vocab] parsing vocab.rs ...")
    vocab_lists = parse_vocab_rs(VOCAB_RS)

    token_sets:        dict[str, set[str]] = {}
    corpus_texts:      dict[str, str]      = {}
    canonical_counter: Counter             = Counter()
    folger_play_texts: dict[str, str]      = {}

    guten_tokens, guten_text, guten_counts = load_guten()
    token_sets["guten-complete-2026"]   = guten_tokens
    corpus_texts["guten-complete-2026"] = guten_text
    canonical_counter.update(guten_counts)

    folger_tokens, folger_text, folger_play_texts, folger_counts = load_folger()
    token_sets["folger-complete-2026"]   = folger_tokens
    corpus_texts["folger-complete-2026"] = folger_text
    canonical_counter.update(folger_counts)

    if run_eebo:
        eebo_tokens, eebo_text = load_eebo(full=run_eebo_full)
        token_sets["eebo-period-2026"]   = eebo_tokens
        corpus_texts["eebo-period-2026"] = eebo_text
        # EEBO not added to canonical_counter: frequency counts are oss-moby-2026 only

    active_packs    = list(token_sets.keys())
    canonical_packs = ["guten-complete-2026", "folger-complete-2026"]
    print(f"[packs] active: {active_packs}")
    print(f"[packs] canonical (oss-moby-2026): {canonical_packs}")
    print(f"[folger] {len(folger_play_texts)} plays indexed for play-level provenance")
    print("[folger] building per-play token sets ...")
    play_token_sets = build_play_token_sets(folger_play_texts)
    print(f"[folger] token sets ready for {len(play_token_sets)} plays")

    # All SPL words + character name tokens (for candidate exclusion)
    spl_words: set[str] = set()
    for const_name, _ in VOCAB_CATEGORIES:
        for w in vocab_lists.get(const_name, []):
            spl_words.update(w.lower().split())
    for name in vocab_lists.get("CHARACTER_NAMES", []):
        spl_words.update(name.lower().split())

    # Vocabulary records
    vocab_records: list[dict] = []
    for const_name, spl_class in VOCAB_CATEGORIES:
        for word in vocab_lists.get(const_name, []):
            status, found_in, attested_as, freq = attest_word(
                word, spl_class, token_sets, canonical_counter
            )
            lookup = attested_as if attested_as else word
            plays  = plays_for_word(lookup, play_token_sets) if status == "attested" else []
            vocab_records.append({
                "word":        word,
                "spl_class":   spl_class,
                "const":       const_name,
                "status":      status,
                "attested_in": found_in,
                "attested_as": attested_as,
                "frequency":   freq,
                "plays":       plays,
            })

    # Personae records
    name_records: list[dict] = []
    for name in vocab_lists.get("CHARACTER_NAMES", []):
        status, attested_as, found_in, freq = attest_name(name, corpus_texts, canonical_counter)
        plays = plays_for_name(name, folger_play_texts) if status in ("attested", "attested-alias") else []
        name_records.append({
            "word":        name,
            "spl_class":   "character-name",
            "const":       "CHARACTER_NAMES",
            "status":      status,
            "attested_in": found_in,
            "attested_as": attested_as,
            "frequency":   freq,
            "plays":       plays,
        })

    # §6.4 Candidates
    print("[candidates] extracting vocabulary candidates (§6.4) ...")
    candidate_records = extract_candidates(canonical_counter, spl_words, play_token_sets)

    # Write JSON outputs
    OUT_VOCAB.write_text(json.dumps(vocab_records, indent=2), encoding="utf-8")
    OUT_NAMES.write_text(json.dumps(name_records,  indent=2), encoding="utf-8")
    OUT_CANDIDATES.write_text(json.dumps(candidate_records, indent=2), encoding="utf-8")
    print(f"[output] wrote {OUT_VOCAB.name}  ({len(vocab_records)} entries)")
    print(f"[output] wrote {OUT_NAMES.name}  ({len(name_records)} entries)")
    print(f"[output] wrote {OUT_CANDIDATES.name}  ({len(candidate_records)} entries)")

    # Report
    def pct(n, total): return f"{100*n/total:.1f}%" if total else "n/a"
    def count(recs, s): return sum(1 for r in recs if r["status"] == s)

    v_att = count(vocab_records, "attested");     v_leg = count(vocab_records, "legacy-only")
    n_att = count(name_records,  "attested");     n_ali = count(name_records, "attested-alias")
    n_leg = count(name_records,  "legacy-only")
    total = len(vocab_records) + len(name_records)

    # Lemma rescues
    v_lemma = sum(1 for r in vocab_records if r["status"] == "attested" and r["attested_as"])

    lines = []
    lines.append("SPL Vocabulary & Personae Attestation Report  (oss-moby-2026 v2)")
    lines.append(f"Canonical pack : oss-moby-2026 = guten-complete-2026 + folger-complete-2026")
    lines.append(f"Active packs   : {', '.join(active_packs)}")
    lines.append(f"Play-level     : {len(folger_play_texts)} Folger Digital Texts plays indexed")
    lines.append(f"Lemmatization  : WordNet (NLTK); {v_lemma} words rescued via lemma variant")
    lines.append("")
    lines.append(f"VOCABULARY  {len(vocab_records):>4} words   attested {v_att} ({pct(v_att,len(vocab_records))})   legacy-only {v_leg} ({pct(v_leg,len(vocab_records))})")
    lines.append(f"PERSONAE    {len(name_records):>4} names   attested {n_att} ({pct(n_att,len(name_records))})   alias {n_ali}   legacy-only {n_leg} ({pct(n_leg,len(name_records))})")
    lines.append(f"TOTAL       {total:>4}")
    lines.append(f"CANDIDATES  {len(candidate_records):>4} words proposed for §6.4 expansion table")
    lines.append("")

    for const_name, spl_class in VOCAB_CATEGORIES:
        cat = [r for r in vocab_records if r["const"] == const_name]
        leg = [r for r in cat if r["status"] == "legacy-only"]
        att = [r for r in cat if r["status"] == "attested"]
        lines.append(f"-- {spl_class} ({len(cat)} words, {len(leg)} legacy-only) --")
        for r in att:
            play_str   = ", ".join(r["plays"]) if r["plays"] else "none"
            lemma_note = f"  [lemma: {r['attested_as']}]" if r["attested_as"] else ""
            lines.append(f"  attested     {r['word']:<25}  freq={r['frequency']:>5}  plays({len(r['plays'])}): {play_str}{lemma_note}")
        for r in leg:
            lines.append(f"  LEGACY-ONLY  {r['word']}")
        lines.append("")

    lines.append(f"-- character-name ({len(name_records)} names) --")
    for r in name_records:
        play_str = ", ".join(r["plays"]) if r["plays"] else "none"
        n_plays  = len(r["plays"])
        if r["status"] == "attested":
            lines.append(f"  attested        {r['word']:<35}  freq={r['frequency']:>5}  plays({n_plays}): {play_str}")
        elif r["status"] == "attested-alias":
            lines.append(f"  attested-alias  {r['word']:<35}  -> {r['attested_as']}  freq={r['frequency']:>5}  plays({n_plays}): {play_str}")
        else:
            lines.append(f"  LEGACY-ONLY     {r['word']}")
    lines.append("")

    # §6.4 summary
    lines.append("-- §6.4 candidates (top attested Shakespeare words not in SPL 1.0) --")
    cand_by_cls: dict[str, list[dict]] = {}
    for r in candidate_records:
        cand_by_cls.setdefault(r["suggested_class"], []).append(r)
    for cls in ["positive-adjective", "neutral-adjective", "negative-adjective",
                "positive-noun", "neutral-noun", "negative-noun"]:
        entries = cand_by_cls.get(cls, [])[:15]  # top 15 per class in report
        lines.append(f"  {cls} ({len(cand_by_cls.get(cls, []))} total):")
        for r in entries:
            lines.append(f"    {r['word']:<20}  freq={r['frequency']:>5}  plays({r['n_plays']})")
    lines.append("")

    OUT_REPORT.write_text("\n".join(lines), encoding="utf-8")
    print(f"[output] wrote {OUT_REPORT.name}")

if __name__ == "__main__":
    main()
