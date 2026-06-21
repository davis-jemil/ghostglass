use crate::logger::SharedLogger;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

const FAKE_SNI_POOL: &[&str] = &[
    "ghostglass.internal",
    "api.internal.corp",
    "vault.internal.corp",
    "auth.svc.cluster.local",
    "db-primary.internal",
];

pub async fn start_proxy(addr: &str, logger: SharedLogger) {
    println!("[interceptor::handshake] Binding mock TLS proxy on {addr}");

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            println!("[interceptor::handshake] Bind failed ({e}); falling back to mock mode");
            log_handshake("0.0.0.0:0", fake_sni());
            return;
        }
    };

    println!("[interceptor::handshake] Socket open — ClientHello parser armed on {addr}");

    loop {
        match listener.accept().await {
            Ok((_socket, peer)) => {
                let peer = peer.to_string();
                let sni = fake_sni();
                log_handshake(&peer, sni);
                if let Ok(mut log) = logger.lock() {
                    log.log_connection(&peer, sni);
                }
            }
            Err(e) => {
                println!("[interceptor::handshake] accept() failed: {e}");
            }
        }
    }
}

fn fake_sni() -> &'static str {
    let mut rng = rand::thread_rng();
    FAKE_SNI_POOL[rng.gen_range(0..FAKE_SNI_POOL.len())]
}

fn log_handshake(peer: &str, sni: &str) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    println!(
        "[interceptor::handshake] t={}.{:03} peer={peer} ClientHello (mock): TLSv1.3 SNI={sni} cipher=TLS_AES_256_GCM_SHA384",
        now.as_secs(),
        now.as_millis() % 1000,
    );
}
