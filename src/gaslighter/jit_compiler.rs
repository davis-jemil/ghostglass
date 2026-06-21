use super::procedural_fs;
use crate::entropy;
use crate::logger::SharedLogger;
use rand::Rng;

const HONEYTOKEN_PATTERNS: &[&str] = &[".env", ".key", "id_rsa", "credentials", "shadow", "passwd"];

fn honeytoken_target(cmd: &str) -> Option<&str> {
    cmd.split_whitespace()
        .find(|tok| HONEYTOKEN_PATTERNS.iter().any(|p| tok.contains(p)))
}

/// "Executes" an attacker-supplied shell command against the gaslighter, returning
/// fabricated output convincing enough to keep them digging instead of bailing out.
pub fn execute(command: &str, logger: &SharedLogger) -> String {
    let cmd = command.trim();
    let output = route(cmd, logger);

    if let Ok(mut log) = logger.lock() {
        log.log_command(cmd, &output);
    }
    if let Some(target) = honeytoken_target(cmd) {
        if let Ok(mut log) = logger.lock() {
            log.log_honeytoken(target);
        }
    }

    format!("$ {command}\n{output}")
}

fn route(cmd: &str, logger: &SharedLogger) -> String {
    let first = cmd.split_whitespace().next().unwrap_or("");

    match first {
        "whoami" => return "root".to_string(),
        "ifconfig" => return fake_network(),
        "ip" if matches!(cmd.split_whitespace().nth(1), Some("a") | Some("addr") | Some("address")) => {
            return fake_network();
        }
        "ps" => return fake_processes(),
        "ls" => return fake_ls(cmd, logger),
        "openssl" | "gpg" | "ssh-keygen" | "ssh-copy-id" => {
            if let Some(resp) = entropy::inject_entropy_response(cmd) {
                return resp;
            }
        }
        _ => {}
    }

    if cmd.contains("/etc/passwd") {
        return fake_passwd();
    }
    if cmd.contains("/etc/shadow") {
        return fake_shadow();
    }

    fake_generic(cmd)
}

fn fake_ls(cmd: &str, logger: &SharedLogger) -> String {
    let target = cmd
        .split_whitespace()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .unwrap_or("/root");

    if target.contains("/etc/shadow") {
        return "-rw-r----- 1 root shadow 1278 Jun 21 09:14 /etc/shadow".to_string();
    }
    if target.contains("/etc/passwd") {
        return "-rw-r--r-- 1 root root 2189 Jun 21 09:14 /etc/passwd".to_string();
    }

    let clean_target = target.trim_end_matches('/');
    let prefix = format!("{clean_target}/");

    let rows: Vec<String> = procedural_fs::list_directory(target, logger)
        .into_iter()
        .filter_map(|e| {
            let rel = e.name.strip_prefix(&prefix)?;
            let is_dir = rel.ends_with('/');
            let core = rel.trim_end_matches('/');
            if core.contains('/') {
                return None; // belongs to a deeper level, not this listing
            }
            let perms = if is_dir { "drwxr-xr-x" } else { "-rw-r--r--" };
            Some(format!("{perms} 1 root root {:>8} Jun 21 09:14 {core}", e.size))
        })
        .collect();

    let mut lines = vec![format!("total {}", rows.len() * 4)];
    lines.extend(rows);
    lines.join("\n")
}

fn fake_passwd() -> String {
    [
        "root:x:0:0:root:/root:/bin/bash",
        "daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin",
        "bin:x:2:2:bin:/bin:/usr/sbin/nologin",
        "sys:x:3:3:sys:/dev:/usr/sbin/nologin",
        "sync:x:4:65534:sync:/bin:/bin/sync",
        "games:x:5:60:games:/usr/games:/usr/sbin/nologin",
        "man:x:6:12:man:/var/cache/man:/usr/sbin/nologin",
        "lp:x:7:7:lp:/var/spool/lpd:/usr/sbin/nologin",
        "mail:x:8:8:mail:/var/mail:/usr/sbin/nologin",
        "news:x:9:9:news:/var/spool/news:/usr/sbin/nologin",
        "proxy:x:13:13:proxy:/bin:/usr/sbin/nologin",
        "www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin",
        "backup:x:34:34:backup:/var/backups:/usr/sbin/nologin",
        "nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin",
        "systemd-network:x:100:102:systemd Network Management:/run/systemd:/usr/sbin/nologin",
        "systemd-resolve:x:101:103:systemd Resolver:/run/systemd:/usr/sbin/nologin",
        "sshd:x:103:65534::/run/sshd:/usr/sbin/nologin",
        "postgres:x:104:106:PostgreSQL administrator,,,:/var/lib/postgresql:/bin/bash",
        "redis:x:105:107::/var/lib/redis:/usr/sbin/nologin",
        "deploy:x:1000:1000:deploy:/home/deploy:/bin/bash",
    ]
    .join("\n")
}

fn fake_shadow() -> String {
    [
        "root:$6$rT3kP9mZ$Qc7xVnE2pLk8dRf1tHo9JmZsYwUaXbNc4VqLpRkTjMhGfDsAeBn0:19876:0:99999:7:::",
        "daemon:*:19000:0:99999:7:::",
        "bin:*:19000:0:99999:7:::",
        "sys:*:19000:0:99999:7:::",
        "sshd:*:19000:0:99999:7:::",
        "postgres:$6$wF4tB7nQ$XmRpL9sVzKjEy2cTo6UaHd3Nf1QrWbGtJzVkPmYxLcSe8AoIu:19920:0:99999:7:::",
        "deploy:$6$pQ8nM2vX$Zk5wLrTb3eJhYmNcVqDxRf8sAo1uHpGtKjLnQbWcEz7Yx0iSdA:19980:0:99999:7:::",
        "nobody:*:19000:0:99999:7:::",
    ]
    .join("\n")
}

fn fake_network() -> String {
    let mut rng = rand::thread_rng();
    let a = rng.gen_range(2u8..=254);
    let b = rng.gen_range(2u8..=254);
    let c = rng.gen_range(2u8..=254);
    let mac = (0..6)
        .map(|i| {
            if i == 0 {
                "02".to_string()
            } else {
                format!("{:02x}", rng.gen_range(0u8..=255))
            }
        })
        .collect::<Vec<_>>()
        .join(":");

    [
        "lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536".to_string(),
        "        inet 127.0.0.1  netmask 255.0.0.0".to_string(),
        "        loop  txqueuelen 1000  (Local Loopback)".to_string(),
        String::new(),
        "eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500".to_string(),
        format!("        inet 10.{a}.{b}.{c}  netmask 255.255.255.0  broadcast 10.{a}.{b}.255"),
        format!("        ether {mac}  txqueuelen 0  (Ethernet)"),
    ]
    .join("\n")
}

fn fake_processes() -> String {
    let mut rng = rand::thread_rng();
    let docker_pid = rng.gen_range(700u32..=900);
    let pg_pid = rng.gen_range(1300u32..=1500);
    let bash_pid = rng.gen_range(2800u32..=3100);
    let ps_pid = bash_pid + rng.gen_range(50u32..=200);
    let cpu = rng.gen_range(0u32..=3);

    [
        "USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND".to_string(),
        "root           1  0.0  0.1 168432 11244 ?        Ss   Mon01   0:11 /sbin/init".to_string(),
        "root         412  0.0  0.0  21652  3164 ?        Ss   Mon01   0:03 /usr/sbin/sshd -D".to_string(),
        format!("root        {docker_pid}  0.{cpu}  0.4 712108 36420 ?        Ssl  Mon01   2:47 /usr/bin/dockerd"),
        "root        1190  0.0  0.2 110456  9876 ?        S    Mon01   0:00 nginx: master process".to_string(),
        "www-data    1191  0.0  0.1 110456  6240 ?        S    Mon01   0:00 nginx: worker process".to_string(),
        format!("postgres    {pg_pid}  0.2  1.8 412840 148204 ?       Sl   Mon01   5:12 postgres: writer process"),
        "redis       1488  0.1  0.3  62144  9100 ?        Ssl  Mon01   1:33 redis-server *:6379".to_string(),
        format!("deploy      {bash_pid}  0.0  0.1  18120  5240 pts/0    Ss   09:14   0:00 -bash"),
        format!("root        {ps_pid}  0.0  0.0  17684  2364 pts/0    R+   09:41   0:00 ps aux"),
    ]
    .join("\n")
}

fn fake_generic(cmd: &str) -> String {
    let first = cmd.split_whitespace().next().unwrap_or("");
    match first {
        "" => String::new(),
        "pwd" => "/root".to_string(),
        "hostname" => "ghostglass-prod-01".to_string(),
        "id" => "uid=0(root) gid=0(root) groups=0(root)".to_string(),
        "uname" => "Linux ghostglass-prod-01 5.15.0-92-generic #102-Ubuntu SMP x86_64 GNU/Linux".to_string(),
        "date" => "Sun Jun 21 09:41:13 UTC 2026".to_string(),
        "uptime" => " 09:41:13 up 47 days,  3:12,  1 user,  load average: 0.08, 0.05, 0.01".to_string(),
        "echo" => cmd.splitn(2, ' ').nth(1).unwrap_or("").trim_matches('"').to_string(),
        "cat" | "more" | "less" | "head" | "tail" => {
            let target = cmd.split_whitespace().nth(1).unwrap_or("file");
            format!("{first}: {target}: No such file or directory")
        }
        "cd" | "export" | "mkdir" | "touch" | "rm" | "chmod" | "chown" | "mv" | "cp" | "kill"
        | "ln" | "clear" | "history" => String::new(),
        _ => format!("bash: {first}: command not found"),
    }
}
