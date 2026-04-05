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
        #[serde(default)]
        trace: bool,
    }

    #[derive(Serialize)]
    struct RunResponse {
        output: String,
        error: Option<String>,
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

        let mut logs: Vec<String> = Vec::new();

        // ── Parse phase ───────────────────────────────────────────────────────
        let play = match parse_play(&req.source) {
            Ok(p) => {
                logs.push(format!(
                    "[parse]  OK  —  {} characters,  {} operations,  {} scenes",
                    p.characters.len(), p.ops.len(), p.scene_index.len()
                ));
                p
            }
            Err(e) => {
                logs.push(format!("[parse]  ERROR  —  {}", e));
                return Json(RunResponse {
                    output: String::new(),
                    error: Some(format!("Parse error: {}", e)),
                    ops: 0, ops_run: 0, logs, memory: vec![],
                });
            }
        };

        let total_ops = play.ops.len();
        let do_trace  = req.trace;
        let stdin_bytes = req.stdin.into_bytes();

        // ── Run phase (in a blocking thread with a wall-clock timeout) ────────
        let run_task = tokio::task::spawn_blocking(move || {
            let input  = io::Cursor::new(stdin_bytes);
            let mut output: Vec<u8> = Vec::new();
            let mut interp = Interpreter::new(play, io::BufReader::new(input), &mut output);
            interp.trace     = do_trace;
            interp.max_steps = MAX_STEPS;
            let result = interp.run();
            // Snapshot character memory (sorted by name for stable display)
            let mut memory: Vec<(String, i64)> = interp.state.characters
                .iter()
                .map(|(k, v)| (k.clone(), v.value))
                .collect();
            memory.sort_by(|a, b| a.0.cmp(&b.0));
            (result, interp.state.pc, interp.trace_buf, output, memory)
        });

        let task_result = tokio::time::timeout(
            std::time::Duration::from_secs(TIMEOUT_SECS),
            run_task,
        ).await;

        let (run_result, ops_run, trace_buf, raw_output, memory) = match task_result {
            Ok(Ok(t))  => t,
            Ok(Err(e)) => {
                let msg = format!("internal error: {}", e);
                logs.push(format!("[run]    ERROR  —  {}", msg));
                return Json(RunResponse { output: String::new(), error: Some(msg),
                    ops: total_ops, ops_run: 0, logs, memory: vec![] });
            }
            Err(_) => {
                let msg = format!(
                    "timeout: program did not finish within {} seconds — \
                     interactive programs need all input provided upfront, one response per line",
                    TIMEOUT_SECS
                );
                logs.push(format!("[run]    TIMEOUT  —  {}", msg));
                return Json(RunResponse { output: String::new(), error: Some(msg),
                    ops: total_ops, ops_run: 0, logs, memory: vec![] });
            }
        };

        // Collect trace lines
        if !trace_buf.is_empty() {
            let shown = trace_buf.len().min(MAX_TRACE);
            for line in &trace_buf[..shown] {
                logs.push(format!("[trace]  {}", line));
            }
            if trace_buf.len() > MAX_TRACE {
                logs.push(format!("[trace]  … {} more lines truncated", trace_buf.len() - MAX_TRACE));
            }
        }

        let out_str = String::from_utf8_lossy(
            &raw_output[..raw_output.len().min(MAX_OUTPUT)]
        ).into_owned();

        // Always log memory snapshot
        if !memory.is_empty() {
            logs.push(format!("[memory] {} characters:", memory.len()));
            for (name, val) in &memory {
                logs.push(format!("[memory]   {:20} = {}", name, val));
            }
        }

        match run_result {
            Ok(()) => {
                logs.push(format!("[run]    OK  —  {} / {} ops executed", ops_run, total_ops));
                Json(RunResponse { output: out_str, error: None, ops: total_ops, ops_run, logs, memory })
            }
            Err(e) => {
                logs.push(format!("[run]    STOPPED at op {}  —  {}", ops_run, e));
                Json(RunResponse {
                    output: out_str,
                    error: Some(format!("{}", e)),
                    ops: total_ops, ops_run, logs, memory,
                })
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
  <span>Shakespeare Programming Language interpreter</span>
</header>
<div class="main">
  <div class="pane" style="flex:3">
    <div class="pane-label">Source</div>
    <textarea id="source" spellcheck="false" placeholder="Enter a Shakespeare PL program here…"></textarea>
  </div>
  <div class="divider"></div>
  <div class="right-col">
    <div class="pane">
      <div class="pane-label">Output <span id="decoded-badge" class="decoded-badge" style="display:none">decoded</span></div>
      <div class="output-box" id="output">Press <strong>Run</strong> to execute.</div>
    </div>
    <div class="log-section">
      <div class="log-header" onclick="toggleLog()">
        <span class="log-title">Log</span>
        <span id="log-summary" style="color:#666;font-size:0.7rem"></span>
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
  <button onclick="runSPL()">&#9654; Run</button>
  <span class="status" id="status"></span>
</div>

<script>
// ── State ─────────────────────────────────────────────────────────────────────
let asciiOn  = false;
let traceOn  = false;
let logOpen  = true;
let lastRaw  = '';

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
  renderLogs(['[…] running…']);

  const t0 = performance.now();
  try {
    const resp = await fetch('/run', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source, stdin, trace: traceOn }),
    });
    const data = await resp.json();
    const ms = (performance.now() - t0).toFixed(0);

    renderLogs(data.logs || []);

    if (data.error) {
      // Show partial output (if any) in the output pane; error goes to log only
      outEl.className = data.output ? 'output-box' : 'output-box error';
      if (data.output) {
        showOutput(data.output);
      } else {
        outEl.textContent = data.error;
        badge.style.display = 'none';
      }
      status.textContent = `stopped · op ${data.ops_run}/${data.ops} · ${ms}ms`;
      // Auto-open log so the error and memory are visible
      if (!logOpen) toggleLog();
    } else {
      outEl.className = 'output-box';
      showOutput(data.output);
      status.textContent = `${data.ops_run} ops · ${ms}ms`;
    }
  } catch (e) {
    outEl.className = 'output-box error';
    outEl.textContent = 'Network error: ' + e.message;
    renderLogs([`[error]  Network error: ${e.message}`]);
    status.textContent = 'network error';
  }
}

document.addEventListener('keydown', e => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { e.preventDefault(); runSPL(); }
});
</script>
</body>
</html>"#;
}
