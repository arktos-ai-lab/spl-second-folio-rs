# SPL Folio

A verified Rust interpreter for the [Shakespeare Programming Language](http://shakespeare.sourceforge.net/) (SPL) — an esoteric language in which programs are written as Shakespearean plays, characters are integer variables, and dialogue is executable code.

[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Quick start

```bash
git clone https://github.com/arktos-ai-lab/shakespeare-spl-folio-rs
cd shakespeare-spl-folio-rs
cargo build --release

# Run a program
./target/release/shakespeare run examples/hello.spl

# Interactive web playground
./target/release/shakespeare serve   # → http://localhost:3000
```

---

## What SPL looks like

```
The Rounding of Pi.

Hamlet, a mathematical soul.
Juliet, his constant companion.

                    Act I: Addition.
                    Scene I: The computation begins.

[Enter Hamlet and Juliet]

Hamlet:
 You are the sum of a noble hero and a fine Lord!
 Speak your mind!

[Exeunt]
```

Characters are integer registers (initially 0). The speaker addresses their partner as `you` / `thou`. Value encoding: `sign(noun) × 2^(adjective_count)`.

---

## Language reference

### Value encoding

```
sign(noun) × 2^(number of adjectives)
```

| Expression | Value |
|-----------|-------|
| `nothing` | 0 |
| `a Lord` | +1 |
| `a handsome Lord` | +2 |
| `a beautiful noble angel` | +4 |
| `a pig` | −1 |
| `a fat pig` | −2 |
| `a horrible evil toad` | −4 |

Adjective polarity (positive vs negative) does not affect the sign — only the noun does. Every adjective doubles the magnitude.

### Arithmetic

| Expression | Operation |
|-----------|-----------|
| `the sum of X and Y` | X + Y |
| `the difference between X and Y` | X − Y |
| `the product of X and Y` | X × Y |
| `the quotient between X and Y` | X ÷ Y (truncated toward zero) |
| `the remainder of the quotient between X and Y` | X mod Y |
| `the square of X` | X² |
| `the cube of X` | X³ |
| `the square root of X` | ⌊√X⌋ |
| `the factorial of X` | X! |
| `twice X` | 2X |

### Assignment

```
You are <expr>!        →  set partner's value to expr
Thou art <expr>!       →  (same)
```

### I/O

| Sentence | Action |
|---------|--------|
| `Speak your mind!` | Print partner's value as an **ASCII character** |
| `Open your heart!` | Print partner's value as a **decimal integer** |
| `Open your mind!` | Read one **ASCII character** from stdin into partner (EOF → −1) |
| `Listen to your heart!` | Read one **decimal integer** from stdin into partner |

### Stack

```
Remember yourself.          →  push partner's value onto their stack
Remember <expr>.            →  push expr onto partner's stack
Recall your <anything>.     →  pop from partner's stack into their value
```

### Control flow

```
Am I as good as <expr>?              →  compare speaker == expr
Is <X> greater than <Y>?             →  compare X > Y
Is <X> less than <Y>?                →  compare X < Y
If so, let us proceed to Scene II.   →  conditional jump
If not, let us return to Scene I.    →  conditional jump
Let us proceed to Act II, Scene I.   →  unconditional jump
```

### Stage directions

```
[Enter Hamlet and Ophelia]           →  bring characters on stage
[Exit Hamlet]                        →  remove one character
[Exeunt Hamlet and Ophelia]         →  remove named characters
[Exeunt]                            →  clear the stage
```

---

## Examples

All examples from the original `spl2c` distribution pass:

| Program | Description |
|---------|-------------|
| `hello.spl` | Hello World |
| `fibonacci.spl` | Fibonacci sequence |
| `fibonacci2.spl` | Alternate Fibonacci |
| `primes.spl` | Primes via trial division (reads limit from stdin) |
| `reverse.spl` | Reverse stdin using the stack (send input then click EOF) |
| `guess.spl` | Binary search guessing game 1–1000 (interactive) |
| `shakesbeer.spl` | 99 bottles of beer (turn ASCII decode off to see numbers) |
| `truth_machine.spl` | Prints `1` forever or `0` once |

---

## CLI reference

```bash
shakespeare run   <file.spl>           # run a program
shakespeare run   --trace <file.spl>   # run with op-by-op trace to stderr
shakespeare parse <file.spl>           # parse and dump the flat op list
shakespeare serve [--port 3000]        # start the web playground
```

---

## Character names

Characters must be named after Shakespearean figures. This interpreter recognises the canonical **152-name list** from the original `spl2c` distribution (Hasselström & Åslund, 2001), defined in [`src/vocab.rs`](src/vocab.rs).

---

## Compatibility

| Feature | spl2c | zmbc/shakespearelang | spl-rs |
|---------|-------|----------------------|--------|
| Parser | Custom C | TatSu PEG | Hand-rolled recursive descent |
| Unknown sentences | Error | Error | Skipped (ornamental text) |
| Scene-only goto | Uses current act | Uses current act | Uses current act |
| EOF on char input | −1 | −1 | −1 (CLI); WaitingForInput (web) |
| Re-enter on-stage character | Undefined | No-op | No-op |
| Character vocabulary | ~152 | ~152 | 152 (canonical spl2c list) |

---

## Related projects

- **[zmbc/shakespearelang](https://github.com/zmbc/shakespearelang)** — Python implementation; closest prior art, same spec
- **[spl2c](http://shakespeare.sourceforge.net)** — original 2001 C implementation by Hasselström & Åslund; compiles SPL to C

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) — new SPL example programs are especially welcome, as very few exist anywhere online.

---

## License

MIT
