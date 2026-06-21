use rand::Rng;

const BASE64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(BASE64_TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(BASE64_TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 { BASE64_TABLE[((n >> 6) & 0x3F) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { BASE64_TABLE[(n & 0x3F) as usize] as char } else { '=' });
    }
    out
}

fn wrap_base64(data: &[u8], line_len: usize) -> String {
    let encoded = base64_encode(data);
    encoded
        .chars()
        .collect::<Vec<_>>()
        .chunks(line_len)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Returns a convincing fake PEM key block for the given algorithm (rsa/ec/aes).
pub fn generate_fake_key(algorithm: &str) -> String {
    let (header, footer, byte_len) = match algorithm.to_lowercase().as_str() {
        "rsa" => ("-----BEGIN RSA PRIVATE KEY-----", "-----END RSA PRIVATE KEY-----", 1192),
        "ec" | "ecdsa" => ("-----BEGIN EC PRIVATE KEY-----", "-----END EC PRIVATE KEY-----", 121),
        "aes" => ("-----BEGIN AES PRIVATE KEY-----", "-----END AES PRIVATE KEY-----", 64),
        _ => ("-----BEGIN PRIVATE KEY-----", "-----END PRIVATE KEY-----", 600),
    };

    let body = wrap_base64(&generate_fake_encrypted_blob(byte_len), 64);
    format!("{header}\n{body}\n{footer}")
}

/// Returns random high-entropy bytes that look like encrypted/key material.
pub fn generate_fake_encrypted_blob(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.r#gen::<u8>()).collect()
}

fn fake_hex(rng: &mut impl Rng, bytes: usize, sep: &str) -> String {
    (0..bytes)
        .map(|_| format!("{:02x}", rng.r#gen::<u8>()))
        .collect::<Vec<_>>()
        .join(sep)
}

fn fake_openssl(cmd: &str) -> String {
    let lower = cmd.to_lowercase();
    if lower.contains("genrsa") || lower.contains("rsa") {
        generate_fake_key("rsa")
    } else if lower.contains("ecparam") || lower.contains("ec") {
        generate_fake_key("ec")
    } else if lower.contains("enc") || lower.contains("aes") {
        wrap_base64(&generate_fake_encrypted_blob(192), 76)
    } else if lower.contains("version") {
        "OpenSSL 3.0.13 30 Jan 2024 (Library: OpenSSL 3.0.13 30 Jan 2024)".to_string()
    } else {
        generate_fake_key("rsa")
    }
}

fn fake_gpg(cmd: &str) -> String {
    let lower = cmd.to_lowercase();
    let mut rng = rand::thread_rng();

    if lower.contains("--gen-key") || lower.contains("--full-generate-key") || lower.contains("--quick-gen-key") {
        let fingerprint = fake_hex(&mut rng, 20, " ").to_uppercase();
        format!(
            "gpg: key generation successful\npub   rsa4096 generated [SC]\n      {fingerprint}\nuid           Ghost Admin <admin@ghostglass.internal>\nsub   rsa4096 generated [E]"
        )
    } else if lower.contains("--list-keys") {
        let fingerprint = fake_hex(&mut rng, 20, " ").to_uppercase();
        format!(
            "/root/.gnupg/pubring.kbx\n------------------------\npub   rsa4096 2023-04-11 [SC]\n      {fingerprint}\nuid           [ultimate] Ghost Admin <admin@ghostglass.internal>"
        )
    } else if lower.contains("--encrypt") || lower.contains(" -e ") {
        wrap_base64(&generate_fake_encrypted_blob(256), 64)
    } else {
        "gpg (GnuPG) 2.4.4\nlibgcrypt 1.10.3".to_string()
    }
}

fn fake_ssh_keygen(cmd: &str) -> String {
    let lower = cmd.to_lowercase();
    let key_type = if lower.contains("ed25519") {
        "ED25519"
    } else if lower.contains("ecdsa") {
        "ECDSA"
    } else {
        "RSA"
    };
    let ktype_lower = key_type.to_lowercase();
    let mut rng = rand::thread_rng();
    let fingerprint = fake_hex(&mut rng, 32, ":");

    format!(
        "Generating public/private {key_type} key pair.\nYour identification has been saved in /root/.ssh/id_{ktype_lower}\nYour public key has been saved in /root/.ssh/id_{ktype_lower}.pub\nThe key fingerprint is:\nSHA256:{fingerprint} root@ghostglass-prod-01"
    )
}

fn fake_ssh_copy_id(cmd: &str) -> String {
    let target = cmd.split_whitespace().last().unwrap_or("user@host");
    format!(
        "/usr/bin/ssh-copy-id: INFO: Source of key(s) to be installed: \"/root/.ssh/id_rsa.pub\"\n/usr/bin/ssh-copy-id: INFO: attempting to log in with the new key(s)\nNumber of key(s) added: 1\n\nNow try logging into the machine, with:   \"ssh '{target}'\"\nand check to make sure that only the key(s) you wanted were added."
    )
}

/// If the command is openssl, gpg, or ssh-keygen/ssh-copy-id, fabricates a
/// convincing cryptographic response instead of letting it fall through to
/// the generic "command not found" path.
pub fn inject_entropy_response(cmd: &str) -> Option<String> {
    let first = cmd.split_whitespace().next().unwrap_or("");
    match first {
        "openssl" => Some(fake_openssl(cmd)),
        "gpg" => Some(fake_gpg(cmd)),
        "ssh-keygen" => Some(fake_ssh_keygen(cmd)),
        "ssh-copy-id" => Some(fake_ssh_copy_id(cmd)),
        _ => None,
    }
}
