# Contributing to shakespeare-spl-folio-rs

Thank you for your interest in contributing. This project is a verified reference interpreter for the Shakespeare Programming Language (SPL), built to support research and to serve as the most complete, documented, and tested SPL implementation available. Every contribution — from new example programs to parser fixes to documentation improvements — directly advances that goal.

---

## What contributions are most needed

### 1. New SPL example programs (highest priority)

**Reliable SPL examples are scarce.** The original `spl2c` distribution shipped only eight programs, and very few others have been published or preserved anywhere online. If you have written an SPL program, or can write one, please share it — even small or incomplete programs are valuable as test cases and as illustrations of what the language can express.

Examples are kept in `examples/`. To contribute one:

1. Write your program in a file named `examples/<descriptive-name>.spl`
2. Verify it runs correctly:
   ```
   cargo run -- run examples/your-program.spl
   ```
3. If it requires stdin, document the expected input in a comment at the top of the file or in a companion `.md` file
4. Open a pull request describing what the program computes

**Ideas for programs that do not yet exist:**

| Program | Notes |
|---------|-------|
| FizzBuzz | Output 1–100, replacing multiples of 3 with "Fizz", 5 with "Buzz" |
| ROT13 | Read a string from stdin, output the ROT13 encoding |
| Bubble sort | Sort a short sequence using character stacks |
| GCD / Euclidean algorithm | Two characters, one computation |
| Caesar cipher (encode/decode) | Reads a shift and a string |
| Collatz sequence | Given n, output the Collatz sequence until 1 |
| Decimal to binary | Output the binary representation of an integer |
| Palindrome check | Read a string, output 1 or 0 |
| ASCII art | Fixed output that demonstrates creative use of `Speak your mind` |
| Quine | An SPL program that outputs its own source code |

You do not need to solve all corner cases to contribute. A partially working example with a comment explaining the limitation is still useful.

---

### 2. Parser and interpreter correctness

If you find a program that the original `spl2c` or `zmbc/shakespearelang` runs correctly but `shakespeare-spl-folio-rs` does not, please open an issue with:

- The minimal program that demonstrates the discrepancy
- The expected output (from any other SPL interpreter or from first principles)
- The actual output from `spl-rs`

The test suite lives in `src/parser.rs`, `src/interpreter.rs`, and `src/vocab.rs`. New `#[test]` entries are always welcome.

---

### 3. Vocabulary: missing or misclassified words

SPL noun and adjective lists live in `src/vocab.rs`. If you find a word that:

- appears in `spl2c`'s original word lists but is missing here
- is classified as positive when it should be negative, or vice versa
- causes a computation in one of the example programs to give the wrong result

…please open a PR with the correction and a brief rationale (ideally a reference to the original `spl2c` source or an example program that depends on it).

---

### 4. Documentation

- Corrections or additions to `SPECIFICATION.md` (anything the spec is silent or ambiguous on)
- New entries in `DESIGN.md` (any deliberate departure from the 2001 original)
- Improvements to `README.md`, inline comments, or rustdoc

---

## How to set up and run

```bash
git clone https://github.com/arktos-ai-lab/shakespeare-spl-folio-rs
cd shakespeare-spl-folio-rs
cargo build
cargo test

# Run a program
cargo run -- run examples/hello.spl

# Run with trace output
cargo run -- run --trace examples/fibonacci.spl

# Start the web playground
cargo run -- serve          # → http://localhost:3000
```

Rust stable ≥ 1.70 is sufficient. There are no external system dependencies.

---

## Writing an SPL program — quick reference

SPL programs are structured as Shakespearean plays. Here is the minimum viable structure:

```
A Brief Demonstration.

Romeo, a young man.
Juliet, a young woman.

                    Act I: The only act.

                    Scene I: The only scene.

[Enter Romeo and Juliet]

Romeo:
 You are nothing!

Juliet:
 Open your heart.

[Exeunt]
```

This outputs `0` (Romeo sets Juliet to 0; Juliet outputs Romeo's value as a number).

### Value encoding

A noun phrase encodes an integer: `sign(noun) × 2^(number of adjectives)`.

| Phrase | Value |
|--------|-------|
| `a Lord` | +1 |
| `a pig` | −1 |
| `a handsome Lord` | +2 |
| `a disgusting pig` | −2 |
| `a beautiful noble angel` | +4 |
| `nothing` | 0 |

Positive nouns (angels, kings, heroes, flowers, …) → sign +1  
Negative nouns (pigs, toads, devils, blister, …) → sign −1  
Every adjective (positive or negative) doubles the magnitude.

### Key operations

| What you write | What it does |
|---------------|--------------|
| `You are [expr]!` | Assign `expr` to the character you are speaking to |
| `Speak your mind!` | Output the listener's value as an ASCII character |
| `Open your heart!` | Output the listener's value as a decimal integer |
| `Open your mind!` | Read one ASCII character from stdin into the listener |
| `Listen to your heart!` | Read one decimal integer from stdin into the listener |
| `Remember yourself.` | Push the listener's value onto their stack |
| `Recall your [anything].` | Pop the listener's stack into their value |
| `Am I as good as [expr]?` | Set the global boolean: speaker == expr |
| `If so, let us proceed to Scene II.` | Jump to Scene II if the boolean is true |
| `[Enter Romeo and Juliet]` | Bring characters onto the stage |
| `[Exit Romeo]` | Remove one character from the stage |
| `[Exeunt]` | Remove all characters from the stage |

### Arithmetic

```
the sum of X and Y           →  X + Y
the difference between X and Y  →  X − Y
the product of X and Y       →  X × Y
the quotient between X and Y →  X ÷ Y  (truncated toward zero)
the square of X              →  X²
the cube of X                →  X³
the square root of X         →  ⌊√X⌋
twice X                      →  2X
```

### Tips

- Only **two characters may be on stage** at once for second-person operations to be unambiguous (SPL 1.0 rule; SPL 1.1 adds explicit addressees).
- Characters must be **declared** in the play's header before they can appear on stage.
- Character names must come from the **canonical 152-name list** in `src/vocab.rs` (Hamlet, Juliet, Othello, …).
- Sentences that do not match any operation are treated as **ornamental no-ops** as long as they do not start with a reserved operational head (`You`, `Thou`, `Speak`, `Open`, `Listen`, `Remember`, `Recall`, `Am`, `Is`, `Are`, `Let us`, `We shall`).

---

## Pull request guidelines

- Keep PRs focused: one feature, fix, or example per PR
- For new example programs: include the expected output in the PR description
- For parser/interpreter fixes: include a minimal reproducing program and the expected vs actual output
- Existing tests must continue to pass (`cargo test`)
- No new compiler warnings please (`cargo build` should be clean)

---

## Licence

By contributing, you agree that your contributions will be licensed under the same licence as the project (MIT). SPL example programs you contribute should be original work or clearly in the public domain.
