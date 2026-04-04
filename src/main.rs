use std::fs;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use spl::interpreter::Interpreter;
use spl::parser::parse_play;

#[derive(Parser)]
#[command(
    name = "shakespeare",
    about = "Shakespeare Programming Language interpreter",
    version,
    author,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a Shakespeare PL source file
    Run {
        /// Path to the .spl source file
        file: PathBuf,

        /// Trace each operation to stderr
        #[arg(long)]
        trace: bool,
    },
    /// Parse a file and print the AST (for debugging)
    Parse {
        file: PathBuf,
    },
    /// Check that a file parses without errors
    Check {
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run { file, trace } => {
            let source = fs::read_to_string(&file)
                .with_context(|| format!("reading {:?}", file))?;

            let play = parse_play(&source)
                .with_context(|| format!("parsing {:?}", file))?;

            let stdin  = io::stdin();
            let stdout = io::stdout();
            let mut interp = Interpreter::new(
                play,
                BufReader::new(stdin.lock()),
                stdout.lock(),
            );
            interp.trace = trace;

            interp.run().with_context(|| format!("running {:?}", file))?;
        }

        Command::Parse { file } => {
            let source = fs::read_to_string(&file)
                .with_context(|| format!("reading {:?}", file))?;
            let play = parse_play(&source)
                .with_context(|| format!("parsing {:?}", file))?;

            println!("Title:      {}", play.title);
            println!("Characters: {}", play.characters.len());
            for c in &play.characters {
                println!("  {} — {}", c.name, c.description);
            }
            println!("Operations: {}", play.ops.len());
            println!("Scenes:     {}", play.scene_index.len());
            println!();
            for (i, op) in play.ops.iter().enumerate() {
                println!(
                    "{:4}: [line {:4}] speaker={:15} {:?}",
                    i, op.line,
                    if op.speaker.is_empty() { "<stage>" } else { &op.speaker },
                    op.op,
                );
            }
        }

        Command::Check { file } => {
            let source = fs::read_to_string(&file)
                .with_context(|| format!("reading {:?}", file))?;
            match parse_play(&source) {
                Ok(play) => {
                    println!(
                        "OK — {} characters, {} operations, {} scenes",
                        play.characters.len(),
                        play.ops.len(),
                        play.scene_index.len(),
                    );
                }
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
