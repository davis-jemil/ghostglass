mod interceptor;
mod gaslighter;
mod logger;
mod http_decoy;
mod entropy;
mod profiler;
mod dashboard;
mod web;

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
    let web_dashboard = tokio::spawn(web::start_dashboard("127.0.0.1:3000", logger.clone()));

    interceptor::pqc_adaptor::evaluate();

    let entries = gaslighter::procedural_fs::list_directory("/ghost/hallway", &logger);
    println!("[gaslighter::procedural_fs] Infinite hallway listing for /ghost/hallway:");
    for e in &entries {
        println!("  {:<55} {} bytes", e.name, e.size);
    }

    let output = gaslighter::jit_compiler::execute("ls -la /etc/shadow", &logger);
    println!("[gaslighter::jit_compiler]\n{output}");

    let keygen_output = gaslighter::jit_compiler::execute("ssh-keygen -t rsa -f /root/.ssh/id_rsa", &logger);
    println!("[gaslighter::jit_compiler]\n{keygen_output}");

    println!("Ghostglass Layer 2 active — session logging + HTTP decoy online");

    let log_path = logger.lock().unwrap().path().to_string();
    let profile = profiler::profile_session(&log_path);
    profiler::print_profile(&profile);
    dashboard::print_dashboard(&logger, &profile);

    println!("Ghostglass Layer 3 active — attacker intelligence online");
    println!("Ghostglass Layer 4 active — web dashboard at http://127.0.0.1:3000");
    println!("[ghostglass] Demos complete. TLS proxy listening on 127.0.0.1:8443 — press Ctrl+C to exit.");
    let _ = tokio::join!(proxy, decoy, web_dashboard);
}
