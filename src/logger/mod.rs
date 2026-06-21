use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Shared handle used by every module that needs to record attacker activity.
pub type SharedLogger = Arc<Mutex<SessionLogger>>;

pub struct SessionLogger {
    file: File,
}

impl SessionLogger {
    /// Opens a fresh `logs/session_<timestamp>.log` file for this run.
    pub fn new() -> std::io::Result<Self> {
        fs::create_dir_all("logs")?;
        let started = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let path = format!("logs/session_{started}.log");
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        println!("[logger] Session log opened at {path}");
        Ok(Self { file })
    }

    fn timestamp() -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        format!("{}.{:03}", now.as_secs(), now.as_millis() % 1000)
    }

    fn write_line(&mut self, line: &str) {
        let _ = writeln!(self.file, "{line}");
        let _ = self.file.flush();
    }

    pub fn log_command(&mut self, cmd: &str, output: &str) {
        let line = format!("[{}] COMMAND: {cmd}\n{output}", Self::timestamp());
        println!("{line}");
        self.write_line(&line);
    }

    pub fn log_honeytoken(&mut self, file: &str) {
        let line = format!("[{}] HONEYTOKEN TRIGGERED: {file}", Self::timestamp());
        println!("[ALERT] {line}");
        self.write_line(&format!("[ALERT] {line}"));
    }

    pub fn log_connection(&mut self, peer: &str, sni: &str) {
        let line = format!("[{}] NEW CONNECTION peer={peer} sni={sni}", Self::timestamp());
        println!("{line}");
        self.write_line(&line);
    }

    pub fn log_http_hit(&mut self, peer: &str, path: &str) {
        let line = format!("[{}] HTTP HIT peer={peer} path={path}", Self::timestamp());
        println!("{line}");
        self.write_line(&line);
    }
}
