mod interceptor;
mod gaslighter;
mod logger;
mod http_decoy;

use logger::{SessionLogger, SharedLogger};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    println!("Ghostglass Protocol initializing...");

    let logger: SharedLogger = Arc::new(Mutex::new(
        SessionLogger::new().expect("failed to initialize session logger"),
    ));

    let proxy = tokio::spawn(interceptor::handshake::start_proxy("127.0.0.1:8443", logger.clone()));
    let decoy = tokio::spawn(http_decoy::start_decoy("127.0.0.1:8080", logger.clone()));

    interceptor::pqc_adaptor::evaluate();

    let entries = gaslighter::procedural_fs::list_directory("/ghost/hallway", &logger);
    println!("[gaslighter::procedural_fs] Infinite hallway listing for /ghost/hallway:");
    for e in &entries {
        println!("  {:<55} {} bytes", e.name, e.size);
    }

    let output = gaslighter::jit_compiler::execute("ls -la /etc/shadow", &logger);
    println!("[gaslighter::jit_compiler]\n{output}");

    println!("Ghostglass Layer 2 active — session logging + HTTP decoy online");
    println!("[ghostglass] Demos complete. TLS proxy listening on 127.0.0.1:8443 — press Ctrl+C to exit.");
    let _ = tokio::join!(proxy, decoy);
}
