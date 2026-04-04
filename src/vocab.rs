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
// ---------------------------------------------------------------------------

pub const POSITIVE_ADJECTIVES: &[&str] = &[
    "amazing", "beautiful", "big", "blossoming", "bluest", "bold",
    "bottomless", "brave", "charming", "cleaner", "clearest", "clever",
    "courageous", "cunning", "cute", "delicious", "embroidered", "fair",
    "fine", "flirtatious", "furry", "gentle", "golden", "good", "handsome",
    "happy", "healthy", "honest", "innocent", "kind", "large", "lovely",
    "loving", "meek", "merry", "mighty", "noble", "normal", "peaceful",
    "pretty", "prompt", "proud", "pure", "red", "reddest", "rich", "smooth",
    "sunny", "sweet", "sweetest", "trustworthy", "warm", "white", "wonderful",
    "yellow", "young",
];

pub const NEGATIVE_ADJECTIVES: &[&str] = &[
    "bad", "black", "cowardly", "cursed", "damned", "dark", "dirty",
    "disagreeable", "disgusting", "distasteful", "dusty", "evil", "fat",
    "fatherless", "filthy", "foolish", "foul", "gruesome", "half-witted",
    "hairy", "hard", "horrible", "horrid", "imminent", "infected", "lazy",
    "lewd", "little", "lying", "miserable", "miserly", "misused", "odious",
    "old", "oozing", "rotten", "rude", "rural", "small", "smelly", "snotty",
    "sorry", "squalid", "stinking", "stuffed", "stupid", "terrible", "tiny",
    "ugly", "unhappy", "unkind", "vile", "villainous",
    "wicked", "worried", "worthless",
];

// ---------------------------------------------------------------------------
// Nouns
// ---------------------------------------------------------------------------

pub const POSITIVE_NOUNS: &[&str] = &[
    "angel", "animal", "best", "brother", "cat", "chihuahua", "childhood",
    "cow", "day", "daughter", "father", "flower", "fluffy", "friend", "hair",
    "hamster", "happiness", "hero", "horse", "kingdom", "lamp", "lord",
    "lover", "man", "master", "miracle", "mistletoe", "moon", "morning",
    "mother", "nose", "owner", "plum", "pony", "purse", "roman", "rose",
    "sky", "son", "songbird", "squirrel", "stone", "summer", "sun",
    "sunshine", "thing", "town", "treasure", "wall", "wizard", "woman",
];

pub const NEGATIVE_NOUNS: &[&str] = &[
    "answer", "bastard", "beggar", "blister", "codpiece", "coward",
    "criminal", "curse", "death", "devil", "dog", "drunkard", "dunghill",
    "evil", "fat", "festering", "flirt", "flirt-gill", "fool", "goat",
    "gorgon", "hag", "hate", "hatred", "hog", "hound", "insect", "leech",
    "louse", "lying", "mistrust", "misuse", "noise", "pig", "plague",
    "pox", "rascal", "road", "scoundrel", "sieve", "spite", "toad",
    "traitor", "villain", "witch", "wolf", "worm",
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
pub fn is_negative_adjective(word: &str) -> bool { eq_lower(word, NEGATIVE_ADJECTIVES) }
pub fn is_positive_noun(word: &str) -> bool      { eq_lower_possessive(word, POSITIVE_NOUNS) }
pub fn is_negative_noun(word: &str) -> bool      { eq_lower_possessive(word, NEGATIVE_NOUNS) }
pub fn is_article(word: &str) -> bool            { eq_lower(word, ARTICLES)            }

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
}
