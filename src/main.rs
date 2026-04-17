use std::fs;
use std::io::{self, BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use spl::interpreter::Interpreter;
use spl::parser::{parse_play, parse_play_f1, parse_play_f2};

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
        /// Parse in SPL-F2 mode: ornamental (non-operational) sentences are no-ops
        #[arg(long)]
        f2: bool,
        /// Parse in SPL-F1 (Faithful) mode: F2 extensions plus attestation warnings
        #[arg(long)]
        f1: bool,
    },
    /// Parse a file and print the AST (for debugging)
    Parse {
        file: PathBuf,
        /// Parse in SPL-F2 mode
        #[arg(long)]
        f2: bool,
        /// Parse in SPL-F1 (Faithful) mode: F2 extensions plus attestation warnings
        #[arg(long)]
        f1: bool,
    },
    /// Check that a file parses without errors
    Check {
        file: PathBuf,
        /// Parse in SPL-F2 mode
        #[arg(long)]
        f2: bool,
        /// Parse in SPL-F1 (Faithful) mode: F2 extensions plus attestation warnings
        #[arg(long)]
        f1: bool,
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
        Command::Run { file, trace, f2, f1 } => {
            let source = fs::read_to_string(&file)
                .with_context(|| format!("reading {:?}", file))?;
            let play = if f1 { parse_play_f1(&source) }
                       else if f2 { parse_play_f2(&source) }
                       else { parse_play(&source) }
                .with_context(|| format!("parsing {:?}", file))?;
            if !play.warnings.is_empty() {
                for w in &play.warnings {
                    eprintln!("[F1 warn] {}", w);
                }
            }
            let stdin  = io::stdin();
            let stdout = io::stdout();
            let mut interp = Interpreter::new(
                play,
                BufReader::new(stdin.lock()),
                stdout.lock(),
            );
            interp.trace = trace;
            interp.eof_as_minus_one = true;  // CLI: match original spl2c EOF=-1 behaviour
            interp.run().with_context(|| format!("running {:?}", file))?;
        }

        Command::Parse { file, f2, f1 } => {
            let source = fs::read_to_string(&file)
                .with_context(|| format!("reading {:?}", file))?;
            let play = if f1 { parse_play_f1(&source) }
                       else if f2 { parse_play_f2(&source) }
                       else { parse_play(&source) }
                .with_context(|| format!("parsing {:?}", file))?;
            if !play.warnings.is_empty() {
                eprintln!("[F1] {} attestation warning(s):", play.warnings.len());
                for w in &play.warnings { eprintln!("  {}", w); }
            }
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

        Command::Check { file, f2, f1 } => {
            let source = fs::read_to_string(&file)
                .with_context(|| format!("reading {:?}", file))?;
            let parse_result = if f1 { parse_play_f1(&source) }
                               else if f2 { parse_play_f2(&source) }
                               else { parse_play(&source) };
            match parse_result {
                Ok(play) => {
                    println!(
                        "OK — {} characters, {} operations, {} scenes",
                        play.characters.len(), play.ops.len(), play.scene_index.len(),
                    );
                    if !play.warnings.is_empty() {
                        eprintln!("[F1] {} attestation warning(s):", play.warnings.len());
                        for w in &play.warnings { eprintln!("  {}", w); }
                    }
                }
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

    use spl::{interpreter::Interpreter, parser::{parse_play, parse_play_f1, parse_play_f2}};

    #[derive(Deserialize)]
    struct RunRequest {
        source: String,
        stdin: String,
        #[serde(default)]
        trace: bool,
        /// Byte offset into the full output that the client has already displayed.
        /// The response's `output_new` field will contain only output[output_offset..].
        #[serde(default)]
        output_offset: usize,
        /// When true, the interpreter returns -1 on EOF instead of WaitingForInput.
        /// Used for programs that read to EOF (e.g. reverse.spl) rather than interactive ones.
        #[serde(default)]
        send_eof: bool,
        /// Parse in SPL-F2 mode: ornamental sentences are accepted as no-ops.
        #[serde(default)]
        f2: bool,
        /// Parse in SPL-F1 (Faithful) mode: F2 extensions plus attestation warnings.
        #[serde(default)]
        f1: bool,
    }

    #[derive(Serialize)]
    struct RunResponse {
        /// Full output produced so far (all runs accumulated)
        output: String,
        /// Only the output produced since output_offset (the new part for this run)
        output_new: String,
        error: Option<String>,
        /// True when execution stopped because stdin was exhausted (not a real error)
        waiting_for_input: bool,
        /// Total number of operations in the compiled play
        ops: usize,
        /// How many operations were executed before completion or error
        ops_run: usize,
        /// Structured log lines (parse summary + optional trace)
        logs: Vec<String>,
        /// Character memory at the time of completion/error: [(name, value)]
        memory: Vec<(String, i64)>,
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
        const MAX_OUTPUT: usize  = 100_000;
        const MAX_TRACE: usize   = 2_000;
        const MAX_STEPS: u64     = 2_000_000;
        const TIMEOUT_SECS: u64  = 30;

        let output_offset = req.output_offset.min(MAX_OUTPUT);
        let mut logs: Vec<String> = Vec::new();

        // Helper to build an error-early response
        macro_rules! early_err {
            ($msg:expr) => {
                Json(RunResponse {
                    output: String::new(), output_new: String::new(),
                    error: Some($msg), waiting_for_input: false,
                    ops: 0, ops_run: 0, logs, memory: vec![],
                })
            };
        }

        // ── Parse phase ───────────────────────────────────────────────────────
        let parse_result = if req.f1 { parse_play_f1(&req.source) }
                           else if req.f2 { parse_play_f2(&req.source) }
                           else { parse_play(&req.source) };
        let play = match parse_result {
            Ok(p) => {
                let mode_tag = if req.f1 { "F1" } else if req.f2 { "F2" } else { "Classic" };
                logs.push(format!(
                    "[parse]  OK  ({mode_tag})  —  {} characters,  {} operations,  {} scenes",
                    p.characters.len(), p.ops.len(), p.scene_index.len()
                ));
                if !p.warnings.is_empty() {
                    logs.push(format!("[f1]     {} attestation warning(s):", p.warnings.len()));
                    for w in &p.warnings {
                        logs.push(format!("[f1]       {}", w));
                    }
                }
                p
            }
            Err(e) => {
                logs.push(format!("[parse]  ERROR  —  {}", e));
                return early_err!(format!("Parse error: {}", e));
            }
        };

        let total_ops   = play.ops.len();
        let do_trace    = req.trace;
        let do_eof_minus_one = req.send_eof;
        let stdin_bytes = req.stdin.into_bytes();

        // ── Run phase ─────────────────────────────────────────────────────────
        let run_task = tokio::task::spawn_blocking(move || {
            let input  = io::Cursor::new(stdin_bytes);
            let mut output: Vec<u8> = Vec::new();
            let mut interp = Interpreter::new(play, io::BufReader::new(input), &mut output);
            interp.trace            = do_trace;
            interp.max_steps        = MAX_STEPS;
            interp.eof_as_minus_one = do_eof_minus_one;
            let result = interp.run();
            let mut memory: Vec<(String, i64)> = interp.state.characters
                .iter().map(|(k, v)| (k.clone(), v.value)).collect();
            memory.sort_by(|a, b| a.0.cmp(&b.0));
            (result, interp.state.pc, interp.trace_buf, output, memory)
        });

        let (run_result, ops_run, trace_buf, raw_output, memory) =
            match tokio::time::timeout(
                std::time::Duration::from_secs(TIMEOUT_SECS), run_task
            ).await {
                Ok(Ok(t))  => t,
                Ok(Err(e)) => {
                    logs.push(format!("[run]    ERROR  —  internal: {}", e));
                    return early_err!(format!("internal error: {}", e));
                }
                Err(_) => {
                    let msg = format!(
                        "timeout: program did not finish within {} seconds",
                        TIMEOUT_SECS
                    );
                    logs.push(format!("[run]    TIMEOUT  —  {}", msg));
                    return early_err!(msg);
                }
            };

        // Collect trace lines
        if !trace_buf.is_empty() {
            let shown = trace_buf.len().min(MAX_TRACE);
            for line in &trace_buf[..shown] {
                logs.push(format!("[trace]  {}", line));
            }
            if trace_buf.len() > MAX_TRACE {
                logs.push(format!("[trace]  … {} more lines truncated",
                    trace_buf.len() - MAX_TRACE));
            }
        }

        let out_bytes = &raw_output[..raw_output.len().min(MAX_OUTPUT)];
        let out_str   = String::from_utf8_lossy(out_bytes).into_owned();
        let safe_off  = output_offset.min(out_bytes.len());
        let out_new   = String::from_utf8_lossy(&out_bytes[safe_off..]).into_owned();

        // Memory snapshot in log
        if !memory.is_empty() {
            logs.push(format!("[memory] {} characters:", memory.len()));
            for (name, val) in &memory {
                logs.push(format!("[memory]   {:20} = {}", name, val));
            }
        }

        let is_waiting = matches!(run_result,
            Err(ref e) if matches!(e, spl::error::SplError::WaitingForInput));

        if is_waiting {
            logs.push(format!("[run]    waiting for input at op {}", ops_run));
            Json(RunResponse {
                output: out_str, output_new: out_new,
                error: None, waiting_for_input: true,
                ops: total_ops, ops_run, logs, memory,
            })
        } else {
            match run_result {
                Ok(()) => {
                    logs.push(format!("[run]    OK  —  {} / {} ops executed",
                        ops_run, total_ops));
                    Json(RunResponse {
                        output: out_str, output_new: out_new,
                        error: None, waiting_for_input: false,
                        ops: total_ops, ops_run, logs, memory,
                    })
                }
                Err(e) => {
                    logs.push(format!("[run]    STOPPED at op {}  —  {}", ops_run, e));
                    Json(RunResponse {
                        output: out_str, output_new: out_new,
                        error: Some(format!("{}", e)), waiting_for_input: false,
                        ops: total_ops, ops_run, logs, memory,
                    })
                }
            }
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
  header { background: #2c2416; border-bottom: 1px solid #5a4a2a; padding: 10px 20px; display: flex; align-items: center; gap: 16px; flex-shrink: 0; }
  header h1 { font-size: 1.2rem; color: #d4a850; }
  header span { color: #888; font-size: 0.85rem; font-family: monospace; }
  .main { display: flex; flex: 1; overflow: hidden; min-height: 0; }
  .pane { display: flex; flex-direction: column; flex: 1; overflow: hidden; min-height: 0; }
  .pane-label { background: #252018; color: #a09070; font-size: 0.75rem; font-family: monospace; padding: 4px 12px; border-bottom: 1px solid #3a3020; text-transform: uppercase; letter-spacing: 0.08em; flex-shrink: 0; position: relative; }
  textarea { flex: 1; font-family: 'Courier New', monospace; font-size: 0.88rem; padding: 12px; background: #161410; color: #d0c8b8; border: none; resize: none; outline: none; overflow: auto; white-space: pre; }
  .output-box { flex: 1; font-family: 'Courier New', monospace; font-size: 0.88rem; padding: 12px; background: #161410; color: #d0c8b8; overflow: auto; white-space: pre-wrap; word-break: break-all; min-height: 0; }
  .output-box.error { color: #e06060; }
  .divider { width: 4px; background: #3a3020; cursor: col-resize; flex-shrink: 0; }
  /* Right column: output on top, log below */
  .right-col { display: flex; flex-direction: column; flex: 2; overflow: hidden; min-height: 0; }
  .log-section { display: flex; flex-direction: column; border-top: 2px solid #3a3020; flex-shrink: 0; }
  .log-header { background: #1e1a10; color: #806040; font-size: 0.72rem; font-family: monospace; padding: 3px 12px; display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none; }
  .log-header:hover { background: #252018; }
  .log-header .log-title { text-transform: uppercase; letter-spacing: 0.08em; }
  .log-header .log-toggle { margin-left: auto; color: #666; font-size: 0.7rem; }
  .log-body { font-family: 'Courier New', monospace; font-size: 0.75rem; overflow-y: auto; background: #0e0c08; color: #a09060; padding: 6px 12px; max-height: 200px; min-height: 60px; white-space: pre-wrap; word-break: break-all; }
  .log-body.hidden { display: none; }
  .log-line-parse  { color: #70a870; }
  .log-line-run    { color: #70a870; }
  .log-line-error  { color: #e06060; }
  .log-line-trace  { color: #606888; }
  .log-line-trunc  { color: #806040; font-style: italic; }
  .log-line-memory { color: #8878a8; }
  .log-line-f1     { color: #c8a040; }
  /* Controls bar */
  .controls { background: #252018; border-top: 1px solid #3a3020; padding: 8px 12px; display: flex; align-items: center; gap: 10px; flex-wrap: wrap; flex-shrink: 0; }
  .lbl { color: #888; font-size: 0.78rem; font-family: monospace; white-space: nowrap; }
  #stdin { font-family: monospace; font-size: 0.85rem; background: #161410; color: #c8c0a8; border: 1px solid #3a3020; border-radius: 3px; padding: 4px 8px; flex: 1; min-width: 80px; }
  button { background: #5a3e10; color: #f0d890; border: 1px solid #8a6020; border-radius: 3px; padding: 6px 18px; font-family: Georgia, serif; font-size: 0.9rem; cursor: pointer; white-space: nowrap; }
  button:hover { background: #7a5418; }
  button:active { background: #3a2808; }
  button.tog { background: #1a2a1a; border-color: #3a5a3a; color: #90c890; font-size: 0.82rem; padding: 5px 12px; }
  button.tog.on { background: #2a4a2a; border-color: #5a9a5a; color: #b0f0b0; }
  #examples-select { background: #161410; color: #c8c0a8; border: 1px solid #3a3020; border-radius: 3px; padding: 4px 8px; font-family: monospace; font-size: 0.82rem; }
  .status { font-size: 0.75rem; font-family: monospace; color: #888; margin-left: auto; white-space: nowrap; }
  .decoded-badge { position: absolute; right: 8px; top: 2px; font-size: 0.7rem; background: #2a4a2a; color: #80d080; border-radius: 3px; padding: 1px 6px; }
</style>
</head>
<body>
<header>
  <h1>&#127914; SPL Playground</h1>
  <span>Shakespeare Programming Language · spl-rs (Rust)</span>
</header>
<div class="main">
  <div class="pane" style="flex:3">
    <div class="pane-label">Source</div>
    <textarea id="source" spellcheck="false" placeholder="Enter a Shakespeare PL program here…"></textarea>
  </div>
  <div class="divider"></div>
  <div class="right-col">
    <div class="pane">
      <div class="pane-label">Output <span id="decoded-badge" class="decoded-badge" style="display:none">decoded</span><button onclick="copyOutput()" title="Copy output to clipboard" style="padding:1px 8px;font-size:0.7rem;margin-left:8px;background:#1a2a3a;border-color:#3a6a9a;color:#80b0d0;">&#128203; copy</button></div>
      <div class="output-box" id="output">Press <strong>Run</strong> to execute.</div>
    </div>
    <div id="interactive-area" style="display:none; align-items:center; gap:8px; background:#1a1a10; border-top:1px solid #5a4a1a; padding:6px 12px; flex-shrink:0;">
      <span style="color:#d4a850; font-family:monospace; font-size:0.82rem;">&#9654; Program waiting — send response:</span>
      <input id="interactive-input" type="text" maxlength="1"
        style="font-family:monospace; font-size:1rem; background:#0e0c08; color:#f0e0a0; border:1px solid #8a6020; border-radius:3px; padding:4px 8px; width:48px; text-align:center;"
        placeholder="&lt; &gt; =" onkeydown="if(event.key==='Enter'){sendInteractive();}">
      <button onclick="sendInteractive()" style="padding:4px 14px; font-size:0.85rem;">Send</button>
      <button onclick="sendEof()" title="Signal end-of-input (EOF = -1) to the program" style="padding:4px 14px; font-size:0.85rem; background:#2a1a0a; border:1px solid #6a4010; color:#c09050;">EOF</button>
      <span style="color:#666; font-size:0.75rem; font-family:monospace;">one char + Enter, or EOF to end input</span>
    </div>
    <div class="log-section">
      <div class="log-header" onclick="toggleLog()">
        <span class="log-title">Log</span>
        <span id="log-summary" style="color:#666;font-size:0.7rem"></span>
        <button onclick="event.stopPropagation();copyLog()" title="Copy log to clipboard" style="padding:1px 8px;font-size:0.68rem;background:#1a1a10;border-color:#4a4020;color:#808060;">&#128203; copy</button>
        <span class="log-toggle" id="log-caret">▲ hide</span>
      </div>
      <div class="log-body" id="log-body"></div>
    </div>
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
  <button class="tog" id="trace-btn" onclick="toggleTrace()" title="Log every operation as it executes. Slow for large programs.">Trace: off</button>
  <button class="tog" id="f2-btn" onclick="toggleF2()" title="SPL-F2 mode: ornamental (non-operational) sentences are accepted as no-ops.">F2: off</button>
  <button class="tog" id="f1-btn" onclick="toggleF1()" title="SPL-F1 (Faithful) mode: F2 extensions plus warnings for legacy-only (unattested) vocabulary.">F1: off</button>
  <button onclick="runSPL()">&#9654; Run</button>
  <span class="status" id="status"></span>
</div>

<script>
// ── State ─────────────────────────────────────────────────────────────────────
let asciiOn        = false;
let traceOn        = false;
let f2On           = false;
let f1On           = false;
let logOpen        = true;
let lastRaw        = '';
// Interactive session state
let sessionStdin   = '';   // accumulated stdin across all sends
let sessionOutput  = '';   // full output accumulated across all sends
let sessionOffset  = 0;    // byte offset into output that has been displayed
let sessionActive  = false; // true while waiting for interactive input

// ── ASCII decode ─────────────────────────────────────────────────────────────
function decodeAsAscii(s) {
  let out = '';
  let i = 0;
  while (i < s.length) {
    let matched = false;
    for (const len of [3, 2, 1]) {
      if (i + len > s.length) continue;
      const chunk = s.slice(i, i + len);
      if (chunk[0] === '-') continue;
      const n = parseInt(chunk, 10);
      if (isNaN(n)) continue;
      if (n >= 32 && n <= 126 || n === 9 || n === 10 || n === 13) {
        out += String.fromCharCode(n);
        i += len; matched = true; break;
      }
      if (len === 1) { out += `[${n}]`; i += 1; matched = true; break; }
    }
    if (!matched) { out += s[i]; i++; }
  }
  return out;
}

function copyOutput() {
  const text = document.getElementById('output').innerText;
  navigator.clipboard.writeText(text).then(() => {
    const btn = event.currentTarget;
    const orig = btn.textContent;
    btn.textContent = '✓ copied';
    setTimeout(() => { btn.textContent = orig; }, 1200);
  });
}

function copyLog() {
  const text = document.getElementById('log').innerText;
  navigator.clipboard.writeText(text).then(() => {
    const btn = event.currentTarget;
    const orig = btn.textContent;
    btn.textContent = '✓ copied';
    setTimeout(() => { btn.textContent = orig; }, 1200);
  });
}

function toggleAscii() {
  asciiOn = !asciiOn;
  const btn = document.getElementById('ascii-btn');
  btn.textContent = asciiOn ? 'ASCII decode: on' : 'ASCII decode: off';
  btn.className = asciiOn ? 'tog on' : 'tog';
  if (lastRaw !== '') showOutput(lastRaw);
}

function toggleTrace() {
  traceOn = !traceOn;
  const btn = document.getElementById('trace-btn');
  btn.textContent = traceOn ? 'Trace: on' : 'Trace: off';
  btn.className = traceOn ? 'tog on' : 'tog';
}

function toggleF2() {
  f2On = !f2On;
  if (f2On && f1On) { f1On = false; syncF1Btn(); }
  const btn = document.getElementById('f2-btn');
  btn.textContent = f2On ? 'F2: on' : 'F2: off';
  btn.className = f2On ? 'tog on' : 'tog';
}

function toggleF1() {
  f1On = !f1On;
  if (f1On && f2On) { f2On = false; syncF2Btn(); }
  const btn = document.getElementById('f1-btn');
  btn.textContent = f1On ? 'F1: on' : 'F1: off';
  btn.className = f1On ? 'tog on' : 'tog';
}

function syncF2Btn() {
  const btn = document.getElementById('f2-btn');
  btn.textContent = f2On ? 'F2: on' : 'F2: off';
  btn.className = f2On ? 'tog on' : 'tog';
}

function syncF1Btn() {
  const btn = document.getElementById('f1-btn');
  btn.textContent = f1On ? 'F1: on' : 'F1: off';
  btn.className = f1On ? 'tog on' : 'tog';
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

// ── Log panel ─────────────────────────────────────────────────────────────────
function toggleLog() {
  logOpen = !logOpen;
  document.getElementById('log-body').className = logOpen ? 'log-body' : 'log-body hidden';
  document.getElementById('log-caret').textContent = logOpen ? '▲ hide' : '▼ show';
}

function renderLogs(logs) {
  const body = document.getElementById('log-body');
  body.innerHTML = '';
  if (!logs || logs.length === 0) {
    body.textContent = '(no log entries)';
    return;
  }
  for (const line of logs) {
    const span = document.createElement('div');
    if (line.startsWith('[parse]') && line.includes('ERROR'))    span.className = 'log-line-error';
    else if (line.startsWith('[run]')  && line.includes('ERROR')) span.className = 'log-line-error';
    else if (line.startsWith('[parse]'))  span.className = 'log-line-parse';
    else if (line.startsWith('[run]'))    span.className = 'log-line-run';
    else if (line.startsWith('[trace]'))  span.className = 'log-line-trace';
    else if (line.startsWith('[memory]')) span.className = 'log-line-memory';
    else if (line.startsWith('[f1]'))     span.className = 'log-line-f1';
    else if (line.includes('truncated')) span.className = 'log-line-trunc';
    span.textContent = line;
    body.appendChild(span);
  }
  // Auto-scroll to bottom so latest entries are visible
  body.scrollTop = body.scrollHeight;
  // Show summary in log header
  const errLines = logs.filter(l => l.includes('ERROR')).length;
  const summary = document.getElementById('log-summary');
  summary.textContent = errLines ? `${errLines} error(s)` : `${logs.length} line(s)`;
  summary.style.color = errLines ? '#e06060' : '#606060';
}

// ── Examples ──────────────────────────────────────────────────────────────────
async function loadExample(name) {
  if (!name) return;
  // guess.spl is interactive: program outputs a guess, you respond with <, >, or =
  // Provide one response per line; = means correct. Example sequence for answer=500:
  const stdinDefaults = { primes: '20', truth_machine: '0', reverse: 'hello',
    guess: '>\n>\n>\n<\n>\n<\n>\n<\n>\n=' };
  document.getElementById('stdin').value = stdinDefaults[name] || '';
  const numericOutput = ['hello', 'primes', 'reverse', 'shakesbeer'];
  asciiOn = numericOutput.includes(name);
  const btn = document.getElementById('ascii-btn');
  btn.textContent = asciiOn ? 'ASCII decode: on' : 'ASCII decode: off';
  btn.className = asciiOn ? 'tog on' : 'tog';
  try {
    const r = await fetch(`/example/${name}`);
    document.getElementById('source').value = r.ok ? await r.text() : `(could not load ${name}.spl)`;
  } catch (e) {
    document.getElementById('source').value = `(network error: ${e.message})`;
  }
  renderLogs([]);
  document.getElementById('status').textContent = '';
}

// ── Interactive send area ─────────────────────────────────────────────────────
function setInteractiveMode(on) {
  sessionActive = on;
  const area = document.getElementById('interactive-area');
  area.style.display = on ? 'flex' : 'none';
  if (on) document.getElementById('interactive-input').focus();
}

function sendInteractive() {
  const inp  = document.getElementById('interactive-input');
  const char = inp.value;   // single char (< > =) — no newline appended
  inp.value  = '';
  if (!char) return;
  // Append to accumulated stdin (no newline — read_char reads one byte at a time)
  sessionStdin += char;
  doRun(document.getElementById('source').value, sessionStdin, sessionOffset);
}

function sendEof() {
  // Re-run with send_eof=true: next EOF read returns -1 instead of waiting.
  // Used for programs like reverse.spl that read until EOF then process results.
  setInteractiveMode(false);
  doRun(document.getElementById('source').value, sessionStdin, sessionOffset, true);
}

// ── Run ───────────────────────────────────────────────────────────────────────
async function runSPL() {
  // Fresh run — reset interactive session state
  sessionStdin  = document.getElementById('stdin').value;
  sessionOutput = '';
  sessionOffset = 0;
  setInteractiveMode(false);
  doRun(document.getElementById('source').value, sessionStdin, 0);
}

async function doRun(source, stdin, outputOffset, sendEof = false) {
  const outEl  = document.getElementById('output');
  const status = document.getElementById('status');
  const badge  = document.getElementById('decoded-badge');

  if (outputOffset === 0) {
    outEl.className  = 'output-box';
    outEl.textContent = 'Running…';
    badge.style.display = 'none';
  }
  status.textContent = '';
  renderLogs(['[…] running…']);

  const t0 = performance.now();
  try {
    const resp = await fetch('/run', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source, stdin, trace: traceOn, output_offset: outputOffset, send_eof: sendEof, f2: f2On, f1: f1On }),
    });
    const data = await resp.json();
    const ms = (performance.now() - t0).toFixed(0);

    renderLogs(data.logs || []);

    // Append new output to our accumulated display
    sessionOutput += (data.output_new || '');
    sessionOffset  = (data.output || '').length;

    if (data.waiting_for_input) {
      // Show what the program produced, then wait for user
      outEl.className = 'output-box';
      showOutput(sessionOutput);
      setInteractiveMode(true);
      status.textContent = `waiting for input · op ${data.ops_run}/${data.ops} · ${ms}ms`;
    } else if (data.error) {
      outEl.className = sessionOutput ? 'output-box' : 'output-box error';
      if (sessionOutput) {
        showOutput(sessionOutput);
      } else {
        outEl.textContent = data.error;
        badge.style.display = 'none';
      }
      setInteractiveMode(false);
      status.textContent = `stopped · op ${data.ops_run}/${data.ops} · ${ms}ms`;
      if (!logOpen) toggleLog();
    } else {
      outEl.className = 'output-box';
      showOutput(sessionOutput);
      setInteractiveMode(false);
      status.textContent = `done · ${data.ops_run} ops · ${ms}ms`;
    }
  } catch (e) {
    outEl.className = 'output-box error';
    outEl.textContent = 'Network error: ' + e.message;
    renderLogs([`[error]  Network error: ${e.message}`]);
    status.textContent = 'network error';
    setInteractiveMode(false);
  }
}

document.addEventListener('keydown', e => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { e.preventDefault(); runSPL(); }
});
</script>
</body>
</html>"#;
}
