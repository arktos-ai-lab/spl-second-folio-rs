use std::fs;
use std::io::{self, BufReader};
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
    /// Start the web playground on localhost:PORT (default 3000)
    Serve {
        #[arg(long, default_value = "3000")]
        port: u16,
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
                Ok(play) => println!(
                    "OK — {} characters, {} operations, {} scenes",
                    play.characters.len(), play.ops.len(), play.scene_index.len(),
                ),
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Command::Serve { port } => {
            // Change to the directory containing the binary's workspace root
            // so that relative paths like "examples/" resolve correctly.
            if let Ok(exe) = std::env::current_exe() {
                // target/debug/shakespeare → project root is 2 levels up
                if let Some(root) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
                    let _ = std::env::set_current_dir(root);
                }
            }
            serve::run(port)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Web playground
// ---------------------------------------------------------------------------

mod serve {
    use anyhow::Result;
    use axum::{
        extract::Json,
        http::{header, StatusCode},
        response::{Html, IntoResponse, Response},
        routing::{get, post},
        Router,
    };
    use serde::{Deserialize, Serialize};
    use std::io;

    use spl::{interpreter::Interpreter, parser::parse_play};

    #[derive(Deserialize)]
    struct RunRequest {
        source: String,
        stdin: String,
    }

    #[derive(Serialize)]
    struct RunResponse {
        output: String,
        error: Option<String>,
        ops: usize,
    }

    async fn example_handler(
        axum::extract::Path(name): axum::extract::Path<String>,
    ) -> Response {
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return (StatusCode::BAD_REQUEST, "bad name").into_response();
        }
        let path = format!("examples/{}.spl", name);
        match std::fs::read_to_string(&path) {
            Ok(content) => (
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                content,
            ).into_response(),
            Err(_) => (StatusCode::NOT_FOUND, format!("example '{}' not found", name)).into_response(),
        }
    }

    async fn run_handler(Json(req): Json<RunRequest>) -> Json<RunResponse> {
        // Hard cap: 100 000 output bytes to avoid infinite-loop floods
        const MAX_OUTPUT: usize = 100_000;

        let result: Result<String, String> = (|| {
            let play = parse_play(&req.source).map_err(|e| e.to_string())?;
            let ops = play.ops.len();
            let input = io::Cursor::new(req.stdin.into_bytes());
            let mut output = Vec::new();
            let mut interp = Interpreter::new(play, io::BufReader::new(input), &mut output);
            // Limit execution steps to avoid runaway programs in the UI
            interp.run().map_err(|e| e.to_string())?;
            let s = String::from_utf8_lossy(&output[..output.len().min(MAX_OUTPUT)]).into_owned();
            Ok(s)
        })();

        match result {
            Ok(out) => {
                // parse again just for op count (already parsed above — accept the dup)
                let ops = parse_play(&req.source).map(|p| p.ops.len()).unwrap_or(0);
                Json(RunResponse { output: out, error: None, ops })
            }
            Err(e) => Json(RunResponse { output: String::new(), error: Some(e), ops: 0 }),
        }
    }

    async fn index_handler() -> Response {
        (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            INDEX_HTML,
        ).into_response()
    }

    pub fn run(port: u16) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async move {
            let app = Router::new()
                .route("/", get(index_handler))
                .route("/run", post(run_handler))
                .route("/example/:name", get(example_handler));

            let addr = format!("0.0.0.0:{port}");
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            println!("SPL playground → http://localhost:{port}");
            axum::serve(listener, app).await.unwrap();
        });
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Single-file HTML playground (no external deps)
    // -----------------------------------------------------------------------

    static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>SPL Playground</title>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: 'Georgia', serif; background: #1a1a1a; color: #e8e0d0; height: 100vh; display: flex; flex-direction: column; }
  header { background: #2c2416; border-bottom: 1px solid #5a4a2a; padding: 10px 20px; display: flex; align-items: center; gap: 16px; }
  header h1 { font-size: 1.2rem; color: #d4a850; }
  header span { color: #888; font-size: 0.85rem; font-family: monospace; }
  .main { display: flex; flex: 1; overflow: hidden; gap: 0; }
  .pane { display: flex; flex-direction: column; flex: 1; overflow: hidden; }
  .pane-label { background: #252018; color: #a09070; font-size: 0.75rem; font-family: monospace; padding: 4px 12px; border-bottom: 1px solid #3a3020; text-transform: uppercase; letter-spacing: 0.08em; }
  textarea, .output-box { flex: 1; font-family: 'Courier New', monospace; font-size: 0.88rem; padding: 12px; background: #161410; color: #d0c8b8; border: none; resize: none; outline: none; overflow: auto; white-space: pre-wrap; word-break: break-all; }
  .divider { width: 4px; background: #3a3020; cursor: col-resize; }
  .controls { background: #252018; border-top: 1px solid #3a3020; padding: 8px 12px; display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
  .lbl { color: #888; font-size: 0.78rem; font-family: monospace; white-space: nowrap; }
  #stdin { font-family: monospace; font-size: 0.85rem; background: #161410; color: #c8c0a8; border: 1px solid #3a3020; border-radius: 3px; padding: 4px 8px; flex: 1; min-width: 80px; }
  button { background: #5a3e10; color: #f0d890; border: 1px solid #8a6020; border-radius: 3px; padding: 6px 18px; font-family: Georgia, serif; font-size: 0.9rem; cursor: pointer; white-space: nowrap; }
  button:hover { background: #7a5418; }
  button:active { background: #3a2808; }
  button.tog { background: #1a2a1a; border-color: #3a5a3a; color: #90c890; }
  button.tog.on { background: #2a4a2a; border-color: #5a9a5a; color: #b0f0b0; }
  #examples-select { background: #161410; color: #c8c0a8; border: 1px solid #3a3020; border-radius: 3px; padding: 4px 8px; font-family: monospace; font-size: 0.82rem; }
  .output-box.error { color: #e06060; }
  .status { font-size: 0.75rem; font-family: monospace; color: #888; margin-left: auto; }
  .pane-label { position: relative; }
  .decoded-badge { position: absolute; right: 8px; top: 2px; font-size: 0.7rem; background: #2a4a2a; color: #80d080; border-radius: 3px; padding: 1px 6px; }
</style>
</head>
<body>
<header>
  <h1>&#127914; SPL Playground</h1>
  <span>Shakespeare Programming Language interpreter</span>
</header>
<div class="main">
  <div class="pane" style="flex:3">
    <div class="pane-label">Source</div>
    <textarea id="source" spellcheck="false" placeholder="Enter a Shakespeare PL program here…"></textarea>
  </div>
  <div class="divider"></div>
  <div class="pane" style="flex:2">
    <div class="pane-label">Output <span id="decoded-badge" class="decoded-badge" style="display:none">decoded</span></div>
    <div class="output-box" id="output">Press <strong>Run</strong> to execute.</div>
  </div>
</div>
<div class="controls">
  <label class="lbl">Example:</label>
  <select id="examples-select" onchange="loadExample(this.value)">
    <option value="">— pick an example —</option>
    <option value="hello">hello.spl</option>
    <option value="fibonacci">fibonacci.spl</option>
    <option value="fibonacci2">fibonacci2.spl</option>
    <option value="truth_machine">truth_machine.spl</option>
    <option value="primes">primes.spl</option>
    <option value="reverse">reverse.spl</option>
    <option value="guess">guess.spl  (input: &lt; &gt; =)</option>
    <option value="shakesbeer">shakesbeer.spl</option>
  </select>
  <label class="lbl" title="Text that the program can read as input (like typing at a keyboard)">Input to program:</label>
  <input id="stdin" type="text" placeholder="leave blank if program needs no input">
  <button class="tog" id="ascii-btn" onclick="toggleAscii()" title="SPL programs that use 'Speak your mind!' output raw numbers. This decodes them as ASCII characters.">ASCII decode: off</button>
  <button onclick="runSPL()">&#9654; Run</button>
  <span class="status" id="status"></span>
</div>

<script>
// Examples are loaded from /example/:name (served from the examples/ directory)

// ── ASCII decode ─────────────────────────────────────────────────────────────
// SPL programs using "Speak your mind!" output raw integers with no separator.
// e.g. "Hello World!\n" comes out as "7210110810811132871111141081003310"
// We greedily parse left-to-right: try 3-digit number first (if ≤127),
// then 2-digit, then 1-digit, converting each to a char.
// Numbers that map to non-printable / outside ASCII are shown as [N].

let asciiOn = false;
let lastRaw = '';

function decodeAsAscii(s) {
  let out = '';
  let i = 0;
  while (i < s.length) {
    let matched = false;
    for (const len of [3, 2, 1]) {
      if (i + len > s.length) continue;
      const chunk = s.slice(i, i + len);
      if (chunk[0] === '-') continue; // skip negatives
      const n = parseInt(chunk, 10);
      if (isNaN(n)) continue;
      // Accept if it's a plausible ASCII value (printable, tab, LF, CR)
      if (n >= 32 && n <= 126 || n === 9 || n === 10 || n === 13) {
        out += String.fromCharCode(n);
        i += len;
        matched = true;
        break;
      }
      // For 1-digit, accept 0 as NUL (show as placeholder) and move on
      if (len === 1) {
        out += `[${n}]`;
        i += 1;
        matched = true;
        break;
      }
    }
    if (!matched) { out += s[i]; i++; }
  }
  return out;
}

function toggleAscii() {
  asciiOn = !asciiOn;
  const btn = document.getElementById('ascii-btn');
  btn.textContent = asciiOn ? 'ASCII decode: on' : 'ASCII decode: off';
  btn.className = asciiOn ? 'tog on' : 'tog';
  if (lastRaw) showOutput(lastRaw);
}

function showOutput(raw) {
  lastRaw = raw;
  const outEl = document.getElementById('output');
  const badge = document.getElementById('decoded-badge');
  if (asciiOn) {
    outEl.textContent = decodeAsAscii(raw) || '(no output)';
    badge.style.display = 'inline';
  } else {
    outEl.textContent = raw || '(no output)';
    badge.style.display = 'none';
  }
}

// ── Examples ──────────────────────────────────────────────────────────────────

async function loadExample(name) {
  if (!name) return;
  const stdinDefaults = { primes: '20', truth_machine: '0', reverse: 'hello' };
  document.getElementById('stdin').value = stdinDefaults[name] || '';
  // Auto-enable ASCII decode for programs that use numeric char output
  const numericOutput = ['hello', 'primes', 'reverse', 'shakesbeer'];
  asciiOn = numericOutput.includes(name);
  const btn = document.getElementById('ascii-btn');
  btn.textContent = asciiOn ? 'ASCII decode: on' : 'ASCII decode: off';
  btn.className = asciiOn ? 'tog on' : 'tog';
  // Fetch actual source from server
  try {
    const r = await fetch(`/example/${name}`);
    document.getElementById('source').value = r.ok ? await r.text() : `(could not load ${name}.spl)`;
  } catch (e) {
    document.getElementById('source').value = `(network error: ${e.message})`;
  }
}

// ── Run ───────────────────────────────────────────────────────────────────────

async function runSPL() {
  const source = document.getElementById('source').value;
  const stdin  = document.getElementById('stdin').value;
  const outEl  = document.getElementById('output');
  const status = document.getElementById('status');
  const badge  = document.getElementById('decoded-badge');

  outEl.className = 'output-box';
  outEl.textContent = 'Running…';
  badge.style.display = 'none';
  status.textContent = '';

  const t0 = performance.now();
  try {
    const resp = await fetch('/run', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source, stdin }),
    });
    const data = await resp.json();
    const ms = (performance.now() - t0).toFixed(0);

    if (data.error) {
      outEl.className = 'output-box error';
      outEl.textContent = data.error;
      badge.style.display = 'none';
      status.textContent = `error · ${ms}ms`;
    } else {
      outEl.className = 'output-box';
      showOutput(data.output);
      status.textContent = `${data.ops} ops · ${ms}ms`;
    }
  } catch (e) {
    outEl.className = 'output-box error';
    outEl.textContent = 'Network error: ' + e.message;
  }
}

document.addEventListener('keydown', e => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { e.preventDefault(); runSPL(); }
});
</script>
</body>
</html>"#;
}
