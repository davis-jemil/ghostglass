mod interceptor;
mod gaslighter;

#[tokio::main]
async fn main() {
    println!("Ghostglass Protocol initializing...");

    interceptor::handshake::start_proxy("127.0.0.1:8443").await;

    interceptor::pqc_adaptor::evaluate();

    let entries = gaslighter::procedural_fs::list_directory("/ghost/hallway");
    println!("[gaslighter::procedural_fs] Infinite hallway listing for /ghost/hallway:");
    for e in &entries {
        println!("  {:<55} {} bytes", e.name, e.size);
    }

    let output = gaslighter::jit_compiler::execute("ls -la /etc/shadow");
    println!("[gaslighter::jit_compiler]\n{output}");
}
