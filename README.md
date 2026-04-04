# shakespeare-spl-folio-rs

A Rust interpreter for the [Shakespeare Programming Language](http://shakespeare.sourceforge.net/) (SPL) — write programs as Shakespearean plays, with characters as variables and dialogue as code.

---

## Install

```bash
git clone https://github.com/arktos-ai-lab/shakespeare-spl-folio-rs
cd shakespeare-spl-folio-rs
cargo build --release
# binary: target/release/shakespeare
```

---

## Usage

```bash
# Run a play
shakespeare run examples/hello_world.spl

# Parse and dump the AST
shakespeare parse examples/hello_world.spl

# Check syntax only
shakespeare check examples/hello_world.spl

# Trace execution (each op printed to stderr)
shakespeare run --trace examples/echo.spl
```

---

## Language overview

### Program structure

```
<Title>.

<Character>, <description>.
<Character>, <description>.

                    Act I: <title>.
                    Scene I: <title>.

[Enter <Character> and <Character>]

<Character>:
  <sentence>!
  <sentence>!

[Exeunt]
```

### Characters as variables

Every character is an integer register (default `0`) plus a stack. Exactly two characters must be on stage for dialogue. The **speaking** character refers to their scene partner with second-person pronouns (`you`, `thou`) and to themselves with first-person pronouns (`I`, `me`).

### Value encoding

```
value = sign(noun) × 2^(adjective_count)
```

| Noun type | Sign | Example |
|-----------|------|---------|
| Positive (horse, rose, sun …) | +1 | `a beautiful horse` → 2 |
| Negative (pig, devil, toad …) | −1 | `a fat lazy pig` → −4 |
| `nothing` | 0 | `nothing` → 0 |

Adjective polarity does not affect the sign — `a beautiful pig` is still −2.

### Arithmetic

```
the sum of X and Y                          →  X + Y
the difference between X and Y             →  X - Y
the product of X and Y                     →  X * Y
the quotient between X and Y               →  X / Y   (truncated toward zero)
the remainder of the quotient between X and Y  →  X % Y
the square of X                            →  X²
the cube of X                              →  X³
the square root of X                       →  floor(√X)
the factorial of X                         →  X!
twice X                                    →  2X
```

### Assignment

```
You are <expr>!      →  partner.value = expr
Thou art <expr>!     →  (same)
```

### I/O

```
Open your mouth!         →  print partner.value as ASCII character
Speak your mind!         →  print partner.value as integer
Listen to your mind!     →  read ASCII character into partner  (EOF → -1)
Listen to your heart!    →  read integer into partner
```

### Stack

```
Remember <expr>!         →  push expr onto partner's stack
Remember yourself!       →  push partner's current value
Recall thy memories!     →  pop top of stack into partner's value
```

### Comparison and control flow

```
Is you greater than X?
Is you less than nothing?        ←  useful for EOF detection (-1 < 0)
Are you and X equal?

If so, <sentence>.
If not, <sentence>.

Let us proceed to scene II.
Let us return to scene I.
Let us proceed to act II.
```

### Stage directions

```
[Enter Hamlet and Ophelia]
[Exit Hamlet]
[Exeunt Hamlet and Ophelia]
[Exeunt]
```

---

## Examples

### Original distribution (source: [shakespeare.sourceforge.net](http://shakespeare.sourceforge.net))

| File | Description |
|------|-------------|
| `hello.spl` | Hello World — the canonical SPL hello world program |
| `fibonacci.spl` | Fibonacci sequence, by Peter Nillius |
| `fibonacci2.spl` | Alternate Fibonacci, by Björn Stenberg and Linus Nielsen Feltzing |
| `primes.spl` | Prime number computation via trial division |
| `reverse.spl` | Reads stdin and prints it reversed using the stack |
| `guess.spl` | Binary-search number guessing game (1–1000), by Jonas Sjöbergh |
| `shakesbeer.spl` | Prints the full "99 bottles of beer" song, by Jonas Sjöbergh |

### Additional examples

| File | Description | Source |
|------|-------------|--------|
| `truth_machine.spl` | Reads an integer; prints `1` forever or `0` once | [esolangs.org/wiki/Shakespeare](https://esolangs.org/wiki/Shakespeare) |

---

## Character names

All characters must be named after Shakespearean figures. The full list recognised by this interpreter is the canonical 152-name set from the original `spl2c` distribution (Hasselström & Åslund, 2001), defined in [`src/vocab.rs`](src/vocab.rs).

---

## Compatibility notes

| Feature | spl2c (original) | zmbc/shakespearelang | spl-rs (this) |
|---------|-----------------|----------------------|---------------|
| Parser | Custom C | TatSu PEG grammar | Hand-rolled recursive descent |
| Unknown sentences | Error | Error | Skipped (flavour text is common) |
| Goto scene-only | Uses current act | Uses current act | Uses current act |
| EOF on char input | Undefined | Returns −1 | Returns −1 |
| Re-entering on-stage character | Undefined | No-op | No-op |
| Character name vocabulary | ~152 | ~152 | 152 (canonical spl2c list) |

---

## Related projects

**[zmbc/shakespearelang](https://github.com/zmbc/shakespearelang)** — Python implementation of the canonical SPL spec. The closest prior art to this project; `spl-rs` targets the same spec and is tested for compatibility.

**[ThyLang](https://github.com/Aruniaaa/ThyLang)** — A Python toy language with Shakespearean-flavored keywords (`hath`, `shouldst`, `printeth`). Note: ThyLang is a *different, independently invented language* — it is not an implementation of the canonical SPL spec. It has no concept of SPL's characters-as-variables, Acts/Scenes, or comparative arithmetic encoding. If you want to run programs written in Shakespeare Programming Language (the 2001 KTH spec), you want this repo, not ThyLang.

**[spl2c](http://shakespeare.sourceforge.net)** — The original 2001 C implementation by Hasselström & Åslund. Compiles SPL to C. The canonical reference for original behaviour.

---

## Running the tests

```bash
cargo test
```

---

## License

MIT
