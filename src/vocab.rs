/// Shakespeare Programming Language vocabulary.
///
/// CHARACTER NAMES
/// ───────────────
/// The canonical 152-name list from the original SPL 1.2.1 specification
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
    "aaron",
    "abbot",
    "abhorson",
    "abraham",
    "achilles",
    "adam",
    "adonis",
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
    "apothecary",
    "archbishop cranmer",
    "archbishop of canterbury",
    "archbishop scroop",
    "archidamus",
    "artemidorus",
    "arthur",
    "arviragus",
    "audrey",
    "autolycus",
    //
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
    "bastard of orleans",
    "bates",
    "beatrice",
    "belarius",
    "benedick",
    "benvolio",
    "bernardo",
    "bertram",
    "bianca",
    "biondello",
    "biron",
    "bishop lincoln",
    "bishop of carlisle",
    "bishop of ely",
    "blanch",
    "blunt",
    "boatswain",
    "bolingbroke",
    "bona",
    "borachio",
    "bottom",
    "boult",
    "boyet",
    "brabantio",
    "brandon",
    "brutus",
    "bushy",
    //
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
    "captain",
    "capucius",
    "capulet",
    "cardinal bourchier",
    "cardinal campeius",
    "cardinal pandulph",
    "cardinal wolsey",
    "carrier",
    "casca",
    "cassandra",
    "cassio",
    "cassius",
    "celia",
    "ceres",
    "cerimon",
    "chamberlain",
    "charles",
    "charles, king of france",
    "charmian",
    "chatillon",
    "chiron",
    "christopher sly",
    "christopher urswick",
    "cicero",
    "cinna",
    "cinna the poet",
    "citizen",
    "citizens",
    "claudio",
    "claudius",
    "cleomenes",
    "cleon",
    "cleopatra",
    "clerk",
    "clerk of chatham",
    "clitus",
    "cloten",
    "cobweb",
    "cominius",
    "commons",
    "conrade",
    "constable of france",
    "constance",
    "cordelia",
    "corin",
    "coriolanus",
    "cornelius",
    "costard",
    "countess",
    "countess of auvergne",
    "court",
    "courtezan",
    "cressida",
    "crier",
    "cromwell",
    "cupid",
    "curan",
    "curio",
    "curtis",
    "cymbeline",
    "cymberline",        // original spec typo — intentionally preserved
    //
    // D
    "dancer",
    "dardanius",
    "daughter of antiochus",
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
    "doctor",
    "doctor butts",
    "doctor caius",
    "dogberry",
    "dolabella",
    "doll tearsheet",
    "domitius enobarus",
    "don adriano de armado",
    "don john",
    "don pedro",
    "donalbain",
    "dorcas",
    "dromio of ephesus",
    "dromio of syracuse",
    "duchess of gloucester",
    "duchess of york",
    "duke",
    "duke of albany",
    "duke of alencon",
    "duke of aumerle",
    "duke of bedford",
    "duke of bourbon",
    "duke of buckingham",
    "duke of burgundy",
    "duke of cornwall",
    "duke of exeter",
    "duke of florence",
    "duke of gloucester",
    "duke of milan",
    "duke of norfolk",
    "duke of orleans",
    "duke of suffolk",
    "duke of surrey",
    "duke of venice",
    "duke of york",
    "duke/earl of somerset",
    "dull",
    "dumain",
    "duncan",
    //
    // E
    "earl of cambridge",
    "earl of douglas",
    "earl of gloucester",
    "earl of kent",
    "earl of northumberland",
    "earl of salisbury",
    "earl of suffolk",
    "earl of surrey",
    "earl of warwick",
    "earl of westmoreland",
    "earl of worcester",
    "earl oxford",
    "edgar",
    "edmond, earl of rutland",
    "edmund",
    "edmund mortimer",
    "edmund of langley",
    "edward poins",
    "egeus",
    "eglamour",
    "egyptian",
    "elbow",
    "eleanor",
    "emilia",
    "english herald",
    "eros",
    "escalus",
    "escanes",
    "essex",
    "euphronius",
    //
    // F
    "fabian",
    "falstaff",
    "fang",
    "father",
    "father john hume",
    "father's ghost",
    "faulconbridge",
    "fenton",
    "ferdinand",
    "feste",
    "flaminius",
    "flavius",
    "fleance",
    "florizel",
    "fluellen",
    "flute",
    "fool",
    "ford",
    "forester",
    "fortinbras",
    "francis",
    "francis feeble",
    "francisca",
    "francisco",
    "frederick",
    "french herald",
    "french soldier",
    "frenchman",
    "friar francis",
    "friar john",
    "friar laurence",
    "friar peter",
    "friar thomas",
    "froth",
    //
    // G
    "gadshill",
    "gallus",
    "gardener",
    "gardiner",
    "garter",
    "general",
    "gentleman",
    "gentlemen",
    "gentlewoman",
    "george bevis",
    "george plantagenet (duke of clarence)",
    "gertrude",
    "glendower",
    "goneril",
    "gonzalo",
    "governor of harfleur",
    "gower",
    "grandpre",
    "gratiano",
    "green",
    "gregory",
    "gremio",
    "griffith",
    "grumio",
    "guard",
    "guiderius",
    "guildenstern",
    //
    // H
    "haberdasher",
    "hamlet",
    "harcourt",
    "hecate",
    "hector",
    "helen",
    "helena",
    "helenus",
    "helicanus",
    "henry iv",
    "henry v",
    "henry vi",
    "henry viii",
    "hermia",
    "hermione",
    "hermonie",          // original spec typo — intentionally preserved
    "hero",
    "hippolyta",
    "holofernes",
    "horatio",
    "hortensio",
    "hortensius",
    "host",
    "hostess quickly",
    "hotspur (henry percy)",
    "hubert de burgh",
    "hymen",
    //
    // I
    "iachimo",
    "iago",
    "imogen",
    "iras",
    "iris",
    "isabella",
    //
    // J
    "jack cade",
    "james gurney",
    "jamy",
    "jaquenetta",
    "jaques (lord)",
    "jaques (son)",
    "jessica",
    "jeweller",
    "joan la pucelle",
    "john holland",
    "john morton",
    "john of gaunt",
    "john of lancaster",
    "john talbot",
    "joseph",
    "julia",
    "juliet",
    "julius caesar",
    "junius brutus",
    "juno",
    "jupiter",
    "justice",
    //
    // K
    "katharine",
    "katherina",
    "king edward iv (plantagenet)",
    "king henry",
    "king john",
    "king lear",
    "king lewis xi",
    "king of france",
    "king phillip",
    "king richard",
    "king richard ii",
    "knight",
    "knights",
    //
    // L
    "lady",
    "lady anne",
    "lady capulet",
    "lady faulconbridge",
    "lady macbeth",
    "lady macduff",
    "lady montague",
    "lady northumberland",
    "lady percy",
    "laertes",
    "lafeu",
    "launce",
    "launcelot gobbo",
    "lavinia",
    "lawyer",
    "le beau",
    "lear",
    "lennox",
    "leonardo",
    "leonato",
    "leonine",
    "leontes",
    "lepidus",
    "lewis",
    "lewis the dauphin",
    "ligarius",
    "lodovico",
    "longaville",
    "lord",
    "lord (earl) rivers",
    "lord abergavenny",
    "lord bardolph",
    "lord berkeley",
    "lord bigot",
    "lord chamberlain",
    "lord chancellor",
    "lord chief justice",
    "lord clifford",
    "lord fitzwater",
    "lord grey",
    "lord hastings",
    "lord lovel",
    "lord marshal",
    "lord mayor of london",
    "lord mowbray",
    "lord ross",
    "lord sands",
    "lord say",
    "lord scales",
    "lord scroop",
    "lord talbot/earl of shrewsbury",
    "lord willoughby",
    "lords",
    "lorenzo",
    "luce",
    "lucentio",
    "lucetta",
    "luciana",
    "lucilius",
    "lucio",
    "lucius",
    "lucullus",
    "lychorida",
    "lymoges",
    "lysander",
    "lysimachus",
    //
    // M
    "macbeth",
    "macduff",
    "macmorris",
    "malcolm",
    "malvolio",
    "mamillius",
    "man",
    "marcellus",
    "marcus andronicus",
    "mardian",
    "margarelon",
    "margaret",
    "margaret jourdain",
    "maria",
    "mariana",
    "marina",
    "mariner",
    "mariners",
    "mark antony",
    "marquess of montague",
    "marquis of dorset",
    "martius",
    "marullus",
    "master",
    "master-gunner",
    "mayor of saint alban's",
    "mayor of york",
    "mecaenas",
    "melun",
    "menas",
    "menecrates",
    "menelaus",
    "menenius agrippa",
    "menteith",
    "mercade",
    "merchant",
    "mercutio",
    "messala",
    "metellus cimber",
    "michael",
    "miranda",
    "mistress ford",
    "mistress overdone",
    "mistress page",
    "montague",
    "montano",
    "montjoy",
    "mopsa",
    "mortimer",
    "morton",
    "moth",
    "mother",
    "mustardseed",
    "mutius",
    "myrmidons",
    //
    // N
    "nathaniel",
    "nerissa",
    "nestor",
    "nicholas",
    "nobleman",
    "norwegian captain",
    "nurse",
    "nym",
    //
    // O
    "oberon",
    "octavia",
    "octavius",
    "octavius caesar",
    "old athenian",
    "old gobbo",
    "old lady",
    "old man",
    "old shepherd",
    "oliver",
    "olivia",
    "ophelia",
    "orlando",
    "orsino",
    "osric",
    "ostler",
    "oswald",
    "othello",
    "outlaws",
    //
    // P
    "page",
    "painter",
    "pandar",
    "pandarus",
    "panthino",
    "pantino",
    "paris",
    "parolles",
    "patience",
    "patrician",
    "patroclus",
    "paulina",
    "peaseblossom",
    "pembroke",
    "perdita",
    "pericles",
    "peter",
    "peter bullcalf",
    "peter of pomfret",
    "peto",
    "petruchio",
    "phebe",
    "philario",
    "philemon",
    "philip",
    "philip the bastard",
    "philo",
    "philostrate",
    "philotus",
    "phrynia",
    "pinch",
    "pindarus",
    "pisanio",
    "pistol",
    "player king",
    "player queen",
    "poet",
    "polixenes",
    "polonius",
    "pompeius",
    "pompey",
    "popilius",
    "portia",
    "posthumus leonatus",
    "priam",
    "prince edward",
    "prince escalus",
    "prince henry",
    "prince humphrey",
    "prince john",
    "prince of arragon",
    "prince of morocco",
    "prince thomas",
    "princes",
    "princess of france",
    "proculeius",
    "prospero",
    "proteus",
    "publius",
    "puck",
    "pursuivant",
    //
    // Q
    "queen",
    "queen elinor",
    "queen elizabeth",
    "queen isabel",
    "queen katharine",
    "queen margaret",
    "quince",
    "quintus",
    //
    // R
    "ralph mouldy",
    "rambures",
    "regan",
    "reignier",
    "reynaldo",
    "richard iii (duke of gloucester)",
    "richard plantagenet (duke of gloucester)",
    "richard plantagenet the younger",
    "richmond (henry vii)",
    "robert shallow",
    "robin",
    "roderigo",
    "roman",
    "roman captain",
    "romeo",
    "rosalind",
    "rosaline",
    "rosencrantz",
    "ross",
    "rugby",
    "rumour",
    //
    // S
    "salanio",
    "salarino",
    "salerio",
    "salisbury",
    "sampson",
    "saturninus",
    "scarus",
    "scout",
    "scribe",
    "scrivener",
    "sebastian",
    "seleucus",
    "sempronius",
    "senator",
    "sentinels",
    "sergeant",
    "servant 1",
    "servant 2",
    "servant 3",
    "servilius",
    "serving-men",
    "seventh citizen",
    "several citizens",
    "sexton",
    "seyton",
    "shallow",
    "shepherd",
    "sheriff of wiltshire",
    "shylock",
    "sicilius leonatus",
    "sicinius velutus",
    "silence",
    "silius",
    "silvia",
    "silvius",
    "simon shadow",
    "simonides",
    "simpcox",
    "simpcox's wife",
    "simple",
    "sir  william glansdale",
    "sir andrew aguecheek",
    "sir anthony denny",
    "sir henry guildford",
    "sir hugh evans",
    "sir humphrey stafford",
    "sir james tyrrel",
    "sir john colville",
    "sir john fastolfe",
    "sir john mortimer",
    "sir john stanley",
    "sir michael",
    "sir nathaniel",
    "sir nicholas vaux",
    "sir oliver martext",
    "sir pierce of exton",
    "sir richard ratcliff",
    "sir robert brakenbury",
    "sir stephen scroop",
    "sir thomas erpingham",
    "sir thomas gargrave",
    "sir thomas grey",
    "sir thomas lovell",
    "sir thomas vaughan",
    "sir toby belch",
    "sir walter herbert",
    "sir william catesby",
    "sir william lucy",
    "sir william stanley",
    "siward",
    "sixth citizen",
    "slender",
    "smith the weaver",
    "snare",
    "snout",
    "snug",
    "solinus",
    "some others",
    "some speak",
    "son",
    "speed",
    "starveling",
    "stephano",
    "strato",
    "surveyor",
    //
    // T
    "tamora",
    "taurus",
    "thaisa",
    "thaliard",
    "the abbot of westminster",
    "the apothecary",
    "the archbishop of canterbury",
    "the duke of milan",
    "the duke of venice",
    "the ghost",
    "thersites",
    "theseus",
    "thieves",
    "thomas horner",
    "thomas mowbray",
    "thomas rotherham",
    "thomas wart",
    "thurio",
    "thyreus",
    "timandra",
    "time",
    "timon",
    "tintinius",
    "titania",
    "titus",
    "titus andronicus",
    "titus lartius",
    "touchstone",
    "townsman",
    "tranio",
    "travellers",
    "travers",
    "trebonius",
    "tribunes",
    "trinculo",
    "troilus",
    "tubal",
    "tullus aufidius",
    "tutor of rutland",
    "tybalt",
    "tyrian sailor",
    //
    // U
    "ulysses",
    "ursula",
    //
    // V
    "valentine",
    "valeria",
    "varrius",
    "varro",
    "vaux",
    "ventidius",
    "venus",
    "verges",
    "vernon",
    "vincentio",
    "viola",
    "virgilia",
    "volsce",
    "voltemand",
    "volumnia",
    "volumnius",
    //
    // W
    "walter whitmore",
    "watch",
    "watchman",
    "widow",
    "william",
    "william page",
    "william stafford",
    "williams",
    "winchester",
    "woodvile",
    //
    // Y
    "young cato",
    "young clifford",
    "young coriolanus",
    "young lucius",
    "young siward",
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
        assert!(n >= 700, "expected 700+ names (SPL 2.0 full personae), got {}", n);
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
