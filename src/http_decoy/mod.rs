use crate::logger::SharedLogger;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const FAKE_ADMIN_PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>System Administration Portal</title>
</head>
<body>
<h1>System Administration Portal</h1>
<form method="POST" action="/login">
  <label for="username">Username</label><br>
  <input type="text" id="username" name="username" autocomplete="off"><br>
  <label for="password">Password</label><br>
  <input type="password" id="password" name="password"><br><br>
  <button type="submit">Log In</button>
</form>
</body>
</html>"#;

/// Binds a fake admin login page and logs every hit as an attacker fingerprint.
pub async fn start_decoy(addr: &str, logger: SharedLogger) {
    println!("[http_decoy] Binding fake admin portal on {addr}");

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            println!("[http_decoy] Bind failed ({e}); HTTP decoy disabled");
            return;
        }
    };

    println!("[http_decoy] Fake admin portal listening on {addr}");

    loop {
        match listener.accept().await {
            Ok((socket, peer)) => {
                tokio::spawn(handle_connection(socket, peer.to_string(), logger.clone()));
            }
            Err(e) => {
                println!("[http_decoy] accept() failed: {e}");
            }
        }
    }
}

async fn handle_connection(mut socket: TcpStream, peer: String, logger: SharedLogger) {
    let mut buf = [0u8; 4096];
    let path = match socket.read(&mut buf).await {
        Ok(n) if n > 0 => request_path(&buf[..n]),
        _ => "/".to_string(),
    };

    if let Ok(mut log) = logger.lock() {
        log.log_http_hit(&peer, &path);
    }

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        FAKE_ADMIN_PAGE.len(),
        FAKE_ADMIN_PAGE
    );

    let _ = socket.write_all(response.as_bytes()).await;
}

fn request_path(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw)
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string()
}
