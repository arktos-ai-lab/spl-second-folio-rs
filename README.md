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

### Hello World (`examples/hello_world.spl`)

The original SPL hello world program. Prints a greeting by encoding ASCII values as arithmetic expressions. Source: [shakespeare.sourceforge.net](http://shakespeare.sourceforge.net)

### Echo (`examples/echo.spl`)

Reads characters from stdin and echoes them to stdout until EOF (detected as −1).

### Truth Machine (`examples/truth_machine.spl`)

Reads an integer: prints `1` forever if the input is `1`, prints `0` once and exits if the input is `0`. A classic esolang benchmark. Source: [esolangs.org/wiki/Shakespeare](https://esolangs.org/wiki/Shakespeare)

### Fibonacci (`examples/fibonacci.spl`)

Reads an integer `n` and prints the first `n` Fibonacci numbers. Demonstrates multi-character programs and loop control across scenes. Source: [github.com/m-fol/Shakespearelang](https://github.com/m-fol/Shakespearelang/blob/main/Fibonacci.spl)

---

## Character names

All characters must be named after Shakespearean figures. The full list recognised by this interpreter is in [`src/vocab.rs`](src/vocab.rs) and includes over 400 names, a strict superset of the original ~152 names from the spl2c reference implementation.

---

## Compatibility notes

| Feature | spl2c (original) | zmbc/shakespearelang | spl-rs (this) |
|---------|-----------------|----------------------|---------------|
| Parser | Custom C | TatSu PEG grammar | Hand-rolled recursive descent |
| Unknown sentences | Error | Error | Skipped (flavour text is common) |
| Goto scene-only | Uses current act | Uses current act | Uses current act |
| EOF on char input | Undefined | Returns −1 | Returns −1 |
| Re-entering on-stage character | Undefined | No-op | No-op |
| Character name vocabulary | ~152 | ~152 | 400+ |

---

## Running the tests

```bash
cargo test
```

---

## License

MIT
