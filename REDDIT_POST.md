# r/ProgrammingLanguages post

**Title:** I came across Shakespeare Programming Language (SPL) and wrote a Rust interpreter for it — repo + formal spec inside

---

I recently fell down a rabbit hole reading about esoteric programming languages and landed on **Shakespeare Programming Language**, a 2001 KTH student project by Hasselström & Åslund. The idea: programs must read as valid Shakespearean dramatic texts. Characters are integer variables. Acts and Scenes are GOTO labels. Arithmetic is expressed as noun phrases — "the sum of a beautiful horse and a pig" is literally `2 + (−1) = 1`.

It's Turing-complete and completely absurd. I had to build an interpreter for it.

**Repo:** https://github.com/arktos-ai-lab/shakespeare-spl-folio-rs

---

## What I built

A Rust interpreter (`shakespeare`) that runs `.spl` files, plus a hand-rolled recursive descent parser that handles the full language:

- Character declarations and stage directions (`[Enter Hamlet and Ophelia]`, `[Exeunt]`)
- Comparative arithmetic (`the difference between the square of X and twice Y`)
- Stack operations (`Remember thyself!`, `Recall thy memories!`)
- Conditional gotos (`If so, let us proceed to scene II.`)
- I/O (both integer and ASCII character modes; EOF returns −1)

All 7 original example programs from the 2001 distribution are included (hello world, primes, Fibonacci, reverse, guessing game, 99 bottles of beer).

---

## Why Rust

The original `spl2c` compiles to C. The only well-maintained modern implementation is [zmbc/shakespearelang](https://github.com/zmbc/shakespearelang) in Python. I wanted a fast, self-contained binary with no runtime dependencies — something you can drop into a pipeline and call as a subprocess.

---

## One thing I learned

The original spec never formally defined which character names are valid. The 2001 release shipped a ~152-name list with no documented criteria. I cross-referenced it against the full Open Source Shakespeare character database, applied consistent filtering rules, and ended up with 400+ names. The methodology is documented in [DESIGN.md](https://github.com/arktos-ai-lab/shakespeare-spl-folio-rs/blob/main/DESIGN.md).

---

## What I'm looking for

- **Bug reports / edge cases** — especially programs that run correctly on spl2c or zmbc but break here
- **Stars** — I'm planning a JOSS submission and they check for community engagement
- **SPL programs** — if you've written anything in SPL I'd love to add it to the examples folder with attribution

Happy to answer questions about the implementation or the language itself.
