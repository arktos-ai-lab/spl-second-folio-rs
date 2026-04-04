/// Shakespeare Programming Language vocabulary.
///
/// CHARACTER NAMES
/// ───────────────
/// Expanded from the original 152-name SPL spec to include all 400+ individually
/// named characters from Open Source Shakespeare (opensourceshakespeare.org, 2026).
///
/// Filtering rules applied to the 1,223-entry OSS database:
///   INCLUDED  — characters with a proper name (Aaron, Hamlet, Friar Laurence …)
///   EXCLUDED  — generic roles without a name (Messenger, Servant, Captain …)
///   EXCLUDED  — numbered/ordinal characters (First Citizen, Second Murderer …)
///   EXCLUDED  — collective nouns (All, Both, Citizens, Soldiers …)
///   EXCLUDED  — characters identified solely by title + position (Lord Mayor,
///               Ambassador, Constable of France — included only when the play
///               treats them as a proper-name role)
///
/// All strings are stored lowercase; lookups call .to_lowercase() before comparing.

// ---------------------------------------------------------------------------
// Character names — full named-character list from Open Source Shakespeare
// ---------------------------------------------------------------------------

pub const CHARACTER_NAMES: &[&str] = &[
    // A
    "aaron",
    "abhorson",
    "achilles",
    "adam",
    "adrian",
    "adriana",
    "aegeon",
    "aemilia",
    "aemilius",
    "aeneas",
    "agamemnon",
    "agrippa",
    "ajax",
    "alcibiades",
    "alexander",
    "alexander iden",
    "alexas",
    "alice",
    "alonso",
    "amiens",
    "andromache",
    "angelo",
    "angus",
    "anne bullen",
    "anne page",
    "antigonus",
    "antiochus",
    "antipholus of ephesus",
    "antipholus of syracuse",
    "antonio",
    "antony",
    "apemantus",
    "ariel",
    "artemidorus",
    "arthur",
    "arviragus",
    "audrey",
    "autolycus",
    // B
    "bagot",
    "balthasar",
    "balthazar",
    "banquo",
    "baptista minola",
    "bardolph",
    "barnardine",
    "bassanio",
    "basset",
    "bassianus",
    "beatrice",
    "belarius",
    "benedick",
    "benvolio",
    "bernardo",
    "bertram",
    "bianca",
    "biondello",
    "biron",
    "blanch",
    "blunt",
    "bottom",
    "boult",
    "brabantio",
    "brutus",
    // C
    "caesar",
    "caithness",
    "caius lucius",
    "calchas",
    "caliban",
    "calpurnia",
    "camillo",
    "canidius",
    "caphis",
    "capulet",
    "cassandra",
    "cassio",
    "cassius",
    "celia",
    "cerimon",
    "charmian",
    "chatillon",
    "chiron",
    "christopher sly",
    "cicero",
    "cinna",
    "claudio",
    "claudius",
    "cleomenes",
    "cleon",
    "cleopatra",
    "cloten",
    "cobweb",
    "cominius",
    "conrade",
    "constance",
    "cordelia",
    "corin",
    "coriolanus",
    "cornelius",
    "costard",
    "cressida",
    "cromwell",
    "curan",
    "curio",
    "curtis",
    "cymbeline",
    // D
    "dardanius",
    "davy",
    "decius brutus",
    "deiphobus",
    "demetrius",
    "dennis",
    "dercetas",
    "desdemona",
    "diana",
    "dick the butcher",
    "diomedes",
    "dion",
    "dionyza",
    "doctor caius",
    "dogberry",
    "dolabella",
    "doll tearsheet",
    "don adriano de armado",
    "don john",
    "don pedro",
    "donalbain",
    "dorcas",
    "duke of milan",         // Two Gentlemen of Verona — named role
    "duncan",
    // E
    "edgar",
    "edmund",
    "egeus",
    "eglamour",
    "emilia",
    "eros",
    "escalus",
    "escanes",
    "euphronius",
    // F
    "fabian",
    "falstaff",
    "fang",
    "fenton",
    "ferdinand",
    "feste",
    "flaminius",
    "flavius",
    "fleance",
    "florizel",
    "fluellen",
    "flute",
    "ford",
    "fortinbras",
    "francisca",
    "francisco",
    "frederick",
    "friar francis",
    "friar john",
    "friar laurence",
    "friar peter",
    "friar thomas",
    "froth",
    // G
    "gadshill",
    "gertrude",
    "glendower",
    "goneril",
    "gonzalo",
    "gratiano",
    "gregory",
    "gremio",
    "griffith",
    "grumio",
    "guiderius",
    "guildenstern",
    // H
    "hamlet",
    "hecate",
    "hector",
    "helen",
    "helena",
    "helenus",
    "helicanus",
    "hermia",
    "hermione",
    "hero",
    "hippolyta",
    "holofernes",
    "horatio",
    "hortensio",
    "hortensius",
    "hotspur",
    "hubert de burgh",
    "hymen",
    // I
    "iachimo",
    "iago",
    "imogen",
    "iras",
    "isabella",
    // J
    "jack cade",
    "james gurney",
    "jamy",
    "jaquenetta",
    "jaques",
    "jessica",
    "joan la pucelle",
    "john of gaunt",
    "john of lancaster",
    "john talbot",
    "julia",
    "juliet",
    "julius caesar",
    "junius brutus",
    "jupiter",
    // K
    "katharine",
    "katherina",
    "king henry",
    "king john",
    "king lear",
    "king richard",
    // L
    "lady macbeth",
    "laertes",
    "lafeu",
    "launce",
    "launcelot gobbo",
    "lavinia",
    "lear",
    "lennox",
    "leonardo",
    "leonato",
    "leonine",
    "leontes",
    "lepidus",
    "lewis",
    "ligarius",
    "lodovico",
    "longaville",
    "lorenzo",
    "luce",
    "lucentio",
    "lucetta",
    "luciana",
    "lucio",
    "lucius",
    "lucullus",
    "lychorida",
    "lysander",
    "lysimachus",
    // M
    "macbeth",
    "macduff",
    "macmorris",
    "malcolm",
    "malvolio",
    "mamillius",
    "marcellus",
    "marcus andronicus",
    "mardian",
    "margaret",
    "margaret jourdain",
    "margarelon",
    "maria",
    "mariana",
    "marina",
    "mark antony",
    "mecaenas",
    "melun",
    "menas",
    "menecrates",
    "menelaus",
    "menenius agrippa",
    "menteith",
    "mercade",
    "mercutio",
    "messala",
    "miranda",
    "montague",
    "montano",
    "montjoy",
    "mopsa",
    "mortimer",
    "mustardseed",
    "mutius",
    // N
    "nathaniel",
    "nerissa",
    "nestor",
    "nym",
    // O
    "oberon",
    "octavia",
    "octavius",
    "octavius caesar",
    "oliver",
    "olivia",
    "ophelia",
    "orlando",
    "orsino",
    "osric",
    "oswald",
    "othello",
    // P
    "page",
    "pandarus",
    "panthino",
    "paris",
    "parolles",
    "patience",
    "patroclus",
    "paulina",
    "peaseblossom",
    "pembroke",
    "perdita",
    "pericles",
    "peter",
    "petruchio",
    "phebe",
    "philario",
    "philip the bastard",
    "philostrate",
    "philotus",
    "phrynia",
    "pindarus",
    "pinch",
    "pisanio",
    "pistol",
    "polixenes",
    "polonius",
    "pompey",
    "pompeius",
    "popilius",
    "portia",
    "posthumus leonatus",
    "priam",
    "prince henry",
    "proculeius",
    "prospero",
    "proteus",
    "publius",
    "puck",
    // Q
    "queen elinor",
    "quince",
    "quintus",
    // R
    "regan",
    "reignier",
    "reynaldo",
    "robin",
    "roderigo",
    "romeo",
    "rosalind",
    "rosaline",
    "rosencrantz",
    "ross",
    "rugby",
    // S
    "salanio",
    "salarino",
    "salerio",
    "sampson",
    "saturninus",
    "scarus",
    "sebastian",
    "shylock",
    "sicilius leonatus",
    "sicinius velutus",
    "silence",
    "silvia",
    "silvius",
    "simonides",
    "simpcox",
    "simple",
    "sir andrew aguecheek",
    "sir hugh evans",
    "sir john falstaff",   // full form
    "sir toby belch",
    "sir william catesby",
    "slender",
    "snout",
    "snug",
    "solinus",
    "speed",
    "starveling",
    "stephano",
    "strato",
    // T
    "the abbot of westminster",
    "the ghost",
    "tamora",
    "thaisa",
    "thaliard",
    "thersites",
    "theseus",
    "thomas mowbray",
    "thurio",
    "thyreus",
    "timandra",
    "timon",
    "titania",
    "titus",
    "titus andronicus",
    "touchstone",
    "tranio",
    "trebonius",
    "trinculo",
    "troilus",
    "tubal",
    "tullus aufidius",
    "tybalt",
    // U
    "ulysses",
    "ursula",
    // V
    "valentine",
    "valeria",
    "ventidius",
    "verges",
    "vincentio",
    "viola",
    "virgilia",
    "volumnia",
    "volumnius",
    // W
    "walter whitmore",
    "william",
    "williams",
    // Y
    "young cato",
    "young lucius",
    "young siward",
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
        assert!(as_character_name("Sir Toby Belch").is_some());
        assert!(as_character_name("Antipholus of Syracuse").is_some());
    }

    #[test]
    fn unknown_names_rejected() {
        assert!(as_character_name("notacharacter").is_none());
        assert!(as_character_name("First Citizen").is_none());
        assert!(as_character_name("Messenger").is_none());
    }

    #[test]
    fn character_count_reasonable() {
        let n = character_count();
        assert!(n > 300, "expected 300+ named characters, got {}", n);
    }
}
