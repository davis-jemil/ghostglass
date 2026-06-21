use tokio::net::TcpListener;

pub async fn start_proxy(addr: &str) {
    println!("[interceptor::handshake] Binding mock TLS proxy on {addr}");
    match TcpListener::bind(addr).await {
        Ok(_listener) => {
            println!("[interceptor::handshake] Socket open — ClientHello parser armed");
            println!("[interceptor::handshake] ClientHello (mock): TLSv1.3 SNI=ghostglass.internal cipher=TLS_AES_256_GCM_SHA384");
        }
        Err(e) => {
            println!("[interceptor::handshake] Bind failed ({e}); falling back to mock mode");
            println!("[interceptor::handshake] ClientHello (mock): TLSv1.3 SNI=ghostglass.internal cipher=TLS_AES_256_GCM_SHA384");
        }
    }
}
