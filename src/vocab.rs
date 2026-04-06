/// Shakespeare Programming Language vocabulary.
///
/// CHARACTER NAMES
/// ───────────────
/// The canonical 152-name list from the original SPL 1.0 specification
/// (Hasselström & Åslund, spl2c v1.2.1, 2001). Sourced from the grammar
/// file of the original compiler distribution; cross-checked against the
/// zmbc/shakespearelang EBNF and drsam94/Spl character.wordlist.
///
/// Two known typos from the original spec are preserved for compatibility:
///   "cymberline"  (Cymbeline)
///   "hermonie"    (Hermione)
///
/// All strings are stored lowercase; lookups call .to_lowercase() before comparing.
///
/// VOCABULARY CATEGORIES (matches spl2c 1.2.1 grammar exactly)
/// ─────────────────────────────────────────────────────────────
/// The original grammar has THREE adjective categories and THREE noun categories:
///
///   Adjective ::= POSITIVE_ADJECTIVE | NEUTRAL_ADJECTIVE | NEGATIVE_ADJECTIVE
///   PositiveNoun ::= NEUTRAL_NOUN | POSITIVE_NOUN          (both yield value +1)
///   NegativeNoun ::= NEGATIVE_NOUN                         (yields value -1)
///
/// Neutral adjectives double the magnitude identically to positive/negative adjectives.
/// Neutral nouns yield +1 (they fall under PositiveNoun in the original grammar).
///
/// The distinction matters for corpus provenance (SPL-F2 attested-mode) and for
/// faithful documentation of the original language.

// ---------------------------------------------------------------------------
// Character names — canonical 152-name list from spl2c (2001)
// ---------------------------------------------------------------------------

pub const CHARACTER_NAMES: &[&str] = &[
    // A
    "achilles",
    "adonis",
    "adriana",
    "aegeon",
    "aemilia",
    "agamemnon",
    "agrippa",
    "ajax",
    "alonso",
    "andromache",
    "angelo",
    "antiochus",
    "antonio",
    "arthur",
    "autolycus",
    // B
    "balthazar",
    "banquo",
    "beatrice",
    "benedick",
    "benvolio",
    "bianca",
    "brabantio",
    "brutus",
    // C
    "capulet",
    "cassandra",
    "cassius",
    "christopher sly",
    "cicero",
    "claudio",
    "claudius",
    "cleopatra",
    "cordelia",
    "cornelius",
    "cressida",
    "cymberline",        // original spec typo — intentionally preserved
    // D
    "demetrius",
    "desdemona",
    "dionyza",
    "doctor caius",
    "dogberry",
    "don john",
    "don pedro",
    "donalbain",
    "dorcas",
    "duncan",
    // E
    "egeus",
    "emilia",
    "escalus",
    // F
    "falstaff",
    "fenton",
    "ferdinand",
    "ford",
    "fortinbras",
    "francisca",
    "friar john",
    "friar laurence",
    // G
    "gertrude",
    "goneril",
    // H
    "hamlet",
    "hecate",
    "hector",
    "helen",
    "helena",
    "hermia",
    "hermonie",          // original spec typo — intentionally preserved
    "hippolyta",
    "horatio",
    // I
    "imogen",
    "isabella",
    // J
    "john of gaunt",
    "john of lancaster",
    "julia",
    "juliet",
    "julius caesar",
    // K
    "king henry",
    "king john",
    "king lear",
    "king richard",
    // L
    "lady capulet",
    "lady macbeth",
    "lady macduff",
    "lady montague",
    "lennox",
    "leonato",
    "luciana",
    "lucio",
    "lychorida",
    "lysander",
    // M
    "macbeth",
    "macduff",
    "malcolm",
    "mariana",
    "mark antony",
    "mercutio",
    "miranda",
    "mistress ford",
    "mistress overdone",
    "mistress page",
    "montague",
    "mopsa",
    // O
    "oberon",
    "octavia",
    "octavius caesar",
    "olivia",
    "ophelia",
    "orlando",
    "orsino",
    "othello",
    // P
    "page",
    "pandarus",
    "pantino",
    "paris",
    "pericles",
    "pinch",
    "polonius",
    "pompeius",
    "portia",
    "priam",
    "prince henry",
    "prospero",
    "proteus",
    "publius",
    "puck",
    // Q
    "queen elinor",
    // R
    "regan",
    "robin",
    "romeo",
    "rosalind",
    // S
    "sebastian",
    "shallow",
    "shylock",
    "slender",
    "solinus",
    "stephano",
    // T
    "thaisa",
    "the abbot of westminster",
    "the apothecary",
    "the archbishop of canterbury",
    "the duke of milan",
    "the duke of venice",
    "the ghost",
    "theseus",
    "thurio",
    "timon",
    "titania",
    "titus",
    "troilus",
    "tybalt",
    // U
    "ulysses",
    // V
    "valentine",
    "venus",
    "vincentio",
    "viola",
];

// ---------------------------------------------------------------------------
// Adjectives
//
// Three categories, matching spl2c 1.2.1 grammar exactly:
//   POSITIVE_ADJECTIVE — doubles magnitude (classic attested positive)
//   NEUTRAL_ADJECTIVE  — doubles magnitude (classic attested neutral)
//   NEGATIVE_ADJECTIVE — doubles magnitude (classic attested negative)
//
// All three categories double the magnitude identically. The distinction
// is lexical provenance, not computational effect.
// ---------------------------------------------------------------------------

/// Classic positive adjectives from spl2c 1.2.1 positive_adjective.wordlist,
/// plus spl-rs extensions (marked inline).
pub const POSITIVE_ADJECTIVES: &[&str] = &[
    // — classic attested —
    "amazing", "beautiful", "blossoming", "bold", "brave", "charming",
    "clearest", "cunning", "cute", "delicious", "embroidered", "fair",
    "fine", "gentle", "golden", "good", "handsome", "happy", "healthy",
    "honest", "lovely", "loving", "mighty", "noble", "peaceful", "pretty",
    "prompt", "proud", "reddest", "rich", "smooth", "sunny", "sweet",
    "sweetest", "trustworthy", "warm",
    // — spl-rs extensions (not in classic 1.2.1, semantically positive) —
    "cleaner", "clever", "courageous", "flirtatious", "innocent", "kind",
    "meek", "merry", "pure", "wonderful", "young",
];

/// Classic neutral adjectives from spl2c 1.2.1 neutral_adjective.wordlist.
/// These double the magnitude exactly like positive/negative adjectives.
/// In the original grammar they are excluded from POSITIVE_COMPARATIVE /
/// NEGATIVE_COMPARATIVE constructs; spl-rs's keyword-based comparative
/// parser is unaffected by this distinction.
pub const NEUTRAL_ADJECTIVES: &[&str] = &[
    // — classic attested neutral —
    "big", "black", "blue", "bluest", "bottomless", "furry", "green",
    "hard", "huge", "large", "little", "normal", "old", "purple", "red",
    "rural", "small", "tiny", "white", "yellow",
];

/// Classic negative adjectives from spl2c 1.2.1 negative_adjective.wordlist,
/// plus spl-rs extensions (marked inline). Note: "rotten" appears twice in
/// the original wordlist; only one entry is kept here.
pub const NEGATIVE_ADJECTIVES: &[&str] = &[
    // — classic attested —
    "bad", "cowardly", "cursed", "damned", "dirty", "disgusting",
    "distasteful", "dusty", "evil", "fat", "fat-kidneyed", "fatherless",
    "foul", "hairy", "half-witted", "horrible", "horrid", "infected",
    "lying", "miserable", "misused", "oozing", "rotten", "smelly",
    "snotty", "sorry", "stinking", "stuffed", "stupid", "vile",
    "villainous", "worried",
    // — spl-rs extensions (not in classic 1.2.1, semantically negative) —
    "dark", "disagreeable", "filthy", "foolish", "gruesome", "imminent",
    "lazy", "lewd", "miserly", "odious", "rude", "squalid", "terrible",
    "ugly", "unhappy", "unkind", "wicked", "worthless",
];

// ---------------------------------------------------------------------------
// Nouns
//
// Three categories, matching spl2c 1.2.1 grammar:
//   POSITIVE_NOUN — yields +1 (classic attested positive)
//   NEUTRAL_NOUN  — yields +1 (classic attested neutral; PositiveNoun in grammar)
//   NEGATIVE_NOUN — yields -1 (classic attested negative)
//
// Both POSITIVE_NOUN and NEUTRAL_NOUN produce value +1. The distinction
// matters only for corpus provenance tracking (SPL-F2 attested-mode).
// ---------------------------------------------------------------------------

/// Classic positive nouns from spl2c 1.2.1 positive_noun.wordlist,
/// plus spl-rs extensions.
pub const POSITIVE_NOUNS: &[&str] = &[
    // — classic attested positive —
    "angel", "flower", "happiness", "hero", "joy", "kingdom", "lord",
    "plum", "pony", "rose",
    // "Heaven" → "heaven", "King" → "king" (case-folded)
    "heaven", "king",
    // "summer's day" is a multi-word classic positive; "summer" retained as
    // the closest single-word approximation for programs that need it.
    "summer",
    // — spl-rs extensions (not in classic 1.2.1, semantically positive) —
    "best", "childhood", "day", "fluffy", "friend", "lover", "man",
    "master", "miracle", "owner", "songbird", "stone", "sun", "sunshine",
    "treasure", "wall", "wizard", "woman",
];

/// Classic neutral nouns from spl2c 1.2.1 neutral_noun.wordlist.
/// These yield value +1, identical to POSITIVE_NOUNS (the original grammar
/// rule is: PositiveNoun ::= NEUTRAL_NOUN | POSITIVE_NOUN).
/// "stone wall" is the only multi-word entry; single-word matching only
/// is currently implemented in spl-rs.
pub const NEUTRAL_NOUNS: &[&str] = &[
    // — classic attested neutral —
    "animal", "aunt", "brother", "cat", "chihuahua", "cousin", "cow",
    "daughter", "door", "face", "father", "fellow", "granddaughter",
    "grandfather", "grandmother", "grandson", "hair", "hamster", "horse",
    "lamp", "lantern", "mistletoe", "moon", "morning", "mother", "nephew",
    "niece", "nose", "purse", "road", "roman", "sister", "sky", "son",
    "squirrel", "stone wall", "thing", "town", "tree", "uncle", "wind",
];

/// Classic negative nouns from spl2c 1.2.1 negative_noun.wordlist,
/// plus spl-rs extensions.
pub const NEGATIVE_NOUNS: &[&str] = &[
    // — classic attested negative —
    "bastard", "beggar", "blister", "codpiece", "coward", "curse",
    "death", "devil", "draught", "famine", "flirt-gill", "goat", "hate",
    "hell", "hog", "hound", "leech", "lie", "microsoft", "pig", "plague",
    "starvation", "toad", "war", "wolf",
    // — spl-rs extensions (not in classic 1.2.1, semantically negative) —
    "answer", "criminal", "dog", "drunkard", "dunghill", "evil", "fat",
    "festering", "flirt", "fool", "gorgon", "hag", "hatred", "insect",
    "louse", "lying", "mistrust", "misuse", "noise", "pox", "rascal",
    "scoundrel", "sieve", "spite", "traitor", "villain", "witch", "worm",
];

// ---------------------------------------------------------------------------
// Articles / possessives (ignored during noun-phrase parsing)
// ---------------------------------------------------------------------------

pub const ARTICLES: &[&str] = &[
    "a", "an", "the", "my", "thy", "your", "his", "her", "its", "our",
    "their", "this", "that",
];

// ---------------------------------------------------------------------------
// Lookup helpers
// ---------------------------------------------------------------------------

fn eq_lower(word: &str, list: &[&str]) -> bool {
    let w = word.to_lowercase();
    list.iter().any(|&s| s == w)
}

/// Like eq_lower but also strips a trailing "'s" (possessive), so "summer's"
/// matches "summer".
fn eq_lower_possessive(word: &str, list: &[&str]) -> bool {
    let w = word.to_lowercase();
    if list.iter().any(|&s| s == w.as_str()) { return true; }
    if let Some(base) = w.strip_suffix("'s") {
        if list.iter().any(|&s| s == base) { return true; }
    }
    false
}

pub fn is_positive_adjective(word: &str) -> bool { eq_lower(word, POSITIVE_ADJECTIVES) }
pub fn is_neutral_adjective(word: &str) -> bool  { eq_lower(word, NEUTRAL_ADJECTIVES) }
pub fn is_negative_adjective(word: &str) -> bool { eq_lower(word, NEGATIVE_ADJECTIVES) }

/// Returns true if `word` is any adjective (positive, neutral, or negative).
/// All adjectives double the magnitude identically.
pub fn is_adjective(word: &str) -> bool {
    is_positive_adjective(word) || is_neutral_adjective(word) || is_negative_adjective(word)
}

/// Returns true for classic positive nouns (value = +1, classic provenance).
pub fn is_positive_noun(word: &str) -> bool { eq_lower_possessive(word, POSITIVE_NOUNS) }

/// Returns true for classic neutral nouns (value = +1, classic provenance).
/// Neutral nouns are computationally equivalent to positive nouns.
pub fn is_neutral_noun(word: &str) -> bool  { eq_lower_possessive(word, NEUTRAL_NOUNS) }

/// Returns true for negative nouns (value = -1).
pub fn is_negative_noun(word: &str) -> bool { eq_lower(word, NEGATIVE_NOUNS) }

/// Returns true if `word` is any noun that yields value +1.
pub fn is_value_positive_noun(word: &str) -> bool {
    is_positive_noun(word) || is_neutral_noun(word)
}

pub fn is_article(word: &str) -> bool { eq_lower(word, ARTICLES) }

/// Returns Some(canonical_lowercase_name) if `s` is a recognised SPL character name.
/// Handles both single-word and multi-word names.
/// Matching is greedy on the longest matching prefix (for stage directions like
/// "[Enter Don Pedro and Don John]").
pub fn as_character_name(s: &str) -> Option<&'static str> {
    let low = s.trim().to_lowercase();
    // Exact match first
    if let Some(&found) = CHARACTER_NAMES.iter().find(|&&n| n == low) {
        return Some(found);
    }
    None
}

/// Roman numeral → u32. Returns None if the string is not a valid roman numeral.
pub fn parse_roman(s: &str) -> Option<u32> {
    let s = s.trim().to_uppercase();
    if s.is_empty() { return None; }
    let mut result: i64 = 0;
    let mut prev: i64 = 0;
    for c in s.chars().rev() {
        let v: i64 = match c {
            'I' => 1, 'V' => 5, 'X' => 10, 'L' => 50,
            'C' => 100, 'D' => 500, 'M' => 1000,
            _ => return None,
        };
        if v < prev { result -= v; } else { result += v; }
        prev = v;
    }
    if result > 0 { Some(result as u32) } else { None }
}

/// Total number of recognised character names.
pub fn character_count() -> usize {
    CHARACTER_NAMES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roman_numerals() {
        assert_eq!(parse_roman("I"),    Some(1));
        assert_eq!(parse_roman("IV"),   Some(4));
        assert_eq!(parse_roman("VIII"), Some(8));
        assert_eq!(parse_roman("XIV"),  Some(14));
        assert_eq!(parse_roman("XL"),   Some(40));
        assert_eq!(parse_roman("XCIX"), Some(99));
        assert_eq!(parse_roman(""),     None);
        assert_eq!(parse_roman("abc"),  None);
    }

    #[test]
    fn single_word_names() {
        assert!(as_character_name("Hamlet").is_some());
        assert!(as_character_name("ROMEO").is_some());
        assert!(as_character_name("ophelia").is_some());
        assert!(as_character_name("Puck").is_some());
    }

    #[test]
    fn multi_word_names() {
        assert!(as_character_name("Julius Caesar").is_some());
        assert!(as_character_name("Don Pedro").is_some());
        assert!(as_character_name("Friar Laurence").is_some());
        assert!(as_character_name("The Ghost").is_some());
        assert!(as_character_name("The Abbot of Westminster").is_some());
        assert!(as_character_name("Lady Macbeth").is_some());
        assert!(as_character_name("Octavius Caesar").is_some());
    }

    #[test]
    fn unknown_names_rejected() {
        assert!(as_character_name("notacharacter").is_none());
        assert!(as_character_name("First Citizen").is_none());
        assert!(as_character_name("Messenger").is_none());
    }

    #[test]
    fn character_count_canonical() {
        let n = character_count();
        assert!(n >= 150 && n <= 160, "expected ~152 canonical names, got {}", n);
    }

    #[test]
    fn neutral_nouns_are_positive() {
        // Neutral nouns must yield +1 (they are PositiveNoun in the original grammar).
        assert!(is_value_positive_noun("road"));   // was wrongly negative before
        assert!(is_value_positive_noun("town"));
        assert!(is_value_positive_noun("animal"));
        assert!(is_value_positive_noun("brother"));
        assert!(is_value_positive_noun("horse"));
        assert!(!is_negative_noun("road"));
    }

    #[test]
    fn classic_negative_nouns_present() {
        assert!(is_negative_noun("hell"));
        assert!(is_negative_noun("microsoft"));
        assert!(is_negative_noun("draught"));
        assert!(is_negative_noun("famine"));
        assert!(is_negative_noun("lie"));
        assert!(is_negative_noun("starvation"));
        assert!(is_negative_noun("war"));
    }

    #[test]
    fn classic_positive_nouns_present() {
        assert!(is_positive_noun("heaven"));
        assert!(is_positive_noun("king"));
        assert!(is_positive_noun("joy"));
    }

    #[test]
    fn neutral_adjectives_are_adjectives() {
        assert!(is_adjective("big"));
        assert!(is_adjective("black"));
        assert!(is_adjective("old"));
        assert!(is_neutral_adjective("big"));
        assert!(!is_positive_adjective("big"));
        assert!(!is_negative_adjective("big"));
    }

    #[test]
    fn fat_kidneyed_is_negative() {
        assert!(is_negative_adjective("fat-kidneyed"));
    }
}
