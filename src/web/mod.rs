use crate::logger::SharedLogger;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>GHOSTGLASS COMMAND CENTER</title>
<style>
  :root {
    --bg: #050806;
    --panel: #0c120f;
    --green: #00ff66;
    --dim-green: #0a3d22;
    --red: #ff3344;
    --grey: #888888;
    --yellow: #e6c200;
    --orange: #ff8800;
  }
  * { box-sizing: border-box; }
  body {
    background: var(--bg);
    color: var(--green);
    font-family: "Courier New", monospace;
    margin: 0;
    padding: 24px;
  }
  h1 {
    text-align: center;
    letter-spacing: 4px;
    text-shadow: 0 0 8px var(--green);
    border-bottom: 1px solid var(--dim-green);
    padding-bottom: 16px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    margin-top: 24px;
  }
  .card {
    background: var(--panel);
    border: 1px solid var(--dim-green);
    border-radius: 6px;
    padding: 16px;
  }
  .card .label {
    color: var(--grey);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 2px;
  }
  .card .value {
    font-size: 28px;
    margin-top: 8px;
  }
  .panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-top: 24px;
  }
  .panel {
    background: var(--panel);
    border: 1px solid var(--dim-green);
    border-radius: 6px;
    padding: 16px;
    min-height: 220px;
  }
  .panel h2 {
    margin: 0 0 12px 0;
    font-size: 14px;
    letter-spacing: 2px;
    color: var(--grey);
  }
  #honeytoken-list {
    list-style: none;
    margin: 0;
    padding: 0;
    color: var(--red);
  }
  #honeytoken-list li {
    padding: 6px 0;
    border-bottom: 1px dashed #311111;
  }
  #honeytoken-list li::before {
    content: "[!] ";
  }
  #terminal {
    background: #000000;
    border-radius: 4px;
    padding: 12px;
    height: 200px;
    overflow-y: auto;
    font-size: 13px;
  }
  #terminal div::before {
    content: "$ ";
    color: var(--grey);
  }
  .skill-Script-Kiddie { color: var(--grey); }
  .skill-Intermediate { color: var(--yellow); }
  .skill-Advanced { color: var(--orange); }
  .skill-APT {
    color: var(--red);
    animation: pulse 1s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }
</style>
</head>
<body>
  <h1>GHOSTGLASS COMMAND CENTER</h1>
  <div class="grid">
    <div class="card"><div class="label">Uptime</div><div class="value" id="uptime">0h 0m 0s</div></div>
    <div class="card"><div class="label">Active Sessions</div><div class="value" id="active-sessions">0</div></div>
    <div class="card"><div class="label">TLS Connections</div><div class="value" id="tls-connections">0</div></div>
    <div class="card"><div class="label">HTTP Hits</div><div class="value" id="http-hits">0</div></div>
    <div class="card"><div class="label">Total Commands</div><div class="value" id="total-commands">0</div></div>
    <div class="card"><div class="label">Skill Assessment</div><div class="value" id="skill-assessment">--</div></div>
  </div>
  <div class="panels">
    <div class="panel">
      <h2>HONEYTOKEN ALERTS</h2>
      <ul id="honeytoken-list"></ul>
    </div>
    <div class="panel">
      <h2>LAST COMMANDS</h2>
      <div id="terminal"></div>
    </div>
  </div>

<script>
let baseUptime = 0;
let lastFetch = Date.now();

function fmtUptime(totalSeconds) {
  const s = Math.floor(totalSeconds);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  return h + "h " + m + "m " + sec + "s";
}

function skillClass(level) {
  return "skill-" + level.replace(/ /g, "-");
}

async function refresh() {
  try {
    const res = await fetch("/api/status");
    const data = await res.json();

    baseUptime = data.uptime_seconds;
    lastFetch = Date.now();

    document.getElementById("active-sessions").textContent = data.active_sessions;
    document.getElementById("tls-connections").textContent = data.tls_connections;
    document.getElementById("http-hits").textContent = data.http_hits;
    document.getElementById("total-commands").textContent = data.total_commands;

    const skillEl = document.getElementById("skill-assessment");
    skillEl.textContent = data.skill_assessment;
    skillEl.className = "value " + skillClass(data.skill_assessment);

    const list = document.getElementById("honeytoken-list");
    list.innerHTML = "";
    data.honeytoken_hits.forEach(function (hit) {
      const li = document.createElement("li");
      li.textContent = hit;
      list.appendChild(li);
    });

    const term = document.getElementById("terminal");
    term.innerHTML = "";
    data.last_commands.forEach(function (cmd) {
      const div = document.createElement("div");
      div.textContent = cmd;
      term.appendChild(div);
    });
    term.scrollTop = term.scrollHeight;
  } catch (e) {
    console.error("status fetch failed", e);
  }
}

function tickUptime() {
  const elapsed = (Date.now() - lastFetch) / 1000;
  document.getElementById("uptime").textContent = fmtUptime(baseUptime + elapsed);
}

refresh();
setInterval(refresh, 3000);
setInterval(tickUptime, 1000);
</script>
</body>
</html>"#;

/// Binds the live operator dashboard: `/` serves the HTML shell, `/api/status`
/// serves a JSON snapshot of the logger's in-memory state for it to poll.
pub async fn start_dashboard(addr: &str, logger: SharedLogger) {
    println!("[web] Binding command center dashboard on {addr}");

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            println!("[web] Bind failed ({e}); web dashboard disabled");
            return;
        }
    };

    println!("[web] Command center listening on {addr}");

    loop {
        match listener.accept().await {
            Ok((socket, _peer)) => {
                tokio::spawn(handle_connection(socket, logger.clone()));
            }
            Err(e) => {
                println!("[web] accept() failed: {e}");
            }
        }
    }
}

async fn handle_connection(mut socket: TcpStream, logger: SharedLogger) {
    let mut buf = [0u8; 4096];
    let path = match socket.read(&mut buf).await {
        Ok(n) if n > 0 => request_path(&buf[..n]),
        _ => "/".to_string(),
    };

    let (status_line, content_type, body) = match path.as_str() {
        "/" => ("200 OK", "text/html; charset=utf-8", DASHBOARD_HTML.to_string()),
        "/api/status" => {
            let status = logger.lock().unwrap().get_status();
            let json = serde_json::to_string(&status).unwrap_or_else(|_| "{}".to_string());
            ("200 OK", "application/json", json)
        }
        _ => ("404 Not Found", "text/plain; charset=utf-8", "Not Found".to_string()),
    };

    let response = format!(
        "HTTP/1.1 {status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );

    let _ = socket.write_all(response.as_bytes()).await;
}

fn request_path(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw)
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string()
}
