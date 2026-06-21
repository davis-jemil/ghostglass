use crate::alerts::AlertConfig;
use crate::profiler::{self, SkillLevel};
use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Shared handle used by every module that needs to record attacker activity.
pub type SharedLogger = Arc<Mutex<SessionLogger>>;

#[derive(Serialize)]
pub struct Status {
    pub uptime_seconds: u64,
    pub active_sessions: u32,
    pub total_commands: usize,
    pub honeytoken_hits: Vec<String>,
    pub skill_assessment: String,
    pub last_commands: Vec<String>,
    pub tls_connections: u32,
    pub http_hits: u32,
}

pub struct SessionLogger {
    file: File,
    path: String,
    started_at: Instant,
    connections: u32,
    tls_connections: u32,
    http_hit_count: u32,
    commands_run: Vec<String>,
    honeytoken_hits: Vec<String>,
    alert_config: Option<AlertConfig>,
    apt_alerted: bool,
}

impl SessionLogger {
    /// Opens a fresh `logs/session_<timestamp>.log` file for this run.
    pub fn new(alert_config: Option<AlertConfig>) -> std::io::Result<Self> {
        fs::create_dir_all("logs")?;
        let started = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let path = format!("logs/session_{started}.log");
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        println!("[logger] Session log opened at {path}");
        Ok(Self {
            file,
            path,
            started_at: Instant::now(),
            connections: 0,
            tls_connections: 0,
            http_hit_count: 0,
            commands_run: Vec::new(),
            honeytoken_hits: Vec::new(),
            alert_config,
            apt_alerted: false,
        })
    }

    fn session_id(&self) -> String {
        profiler::session_id_from_path(&self.path)
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn connection_count(&self) -> u32 {
        self.connections
    }

    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Live, in-memory snapshot of session activity for the web dashboard.
    pub fn get_status(&self) -> Status {
        let skill = crate::profiler::assess_skill(&self.commands_run, &self.honeytoken_hits);
        let start = self.commands_run.len().saturating_sub(5);

        Status {
            uptime_seconds: self.uptime().as_secs(),
            active_sessions: self.connections,
            total_commands: self.commands_run.len(),
            honeytoken_hits: self.honeytoken_hits.clone(),
            skill_assessment: skill.to_string(),
            last_commands: self.commands_run[start..].to_vec(),
            tls_connections: self.tls_connections,
            http_hits: self.http_hit_count,
        }
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
        self.commands_run.push(cmd.to_string());
        let line = format!("[{}] COMMAND: {cmd}\n{output}", Self::timestamp());
        println!("{line}");
        self.write_line(&line);

        if !self.apt_alerted {
            let skill = profiler::assess_skill(&self.commands_run, &self.honeytoken_hits);
            if skill == SkillLevel::Apt {
                self.apt_alerted = true;
                if let Some(config) = self.alert_config.clone() {
                    let session_id = self.session_id();
                    tokio::spawn(async move {
                        config.send_apt_alert(&session_id).await;
                    });
                }
            }
        }
    }

    pub fn log_honeytoken(&mut self, file: &str) {
        self.honeytoken_hits.push(file.to_string());
        let line = format!("[{}] HONEYTOKEN TRIGGERED: {file}", Self::timestamp());
        println!("[ALERT] {line}");
        self.write_line(&format!("[ALERT] {line}"));

        if let Some(config) = self.alert_config.clone() {
            let skill = profiler::assess_skill(&self.commands_run, &self.honeytoken_hits).to_string();
            let session_id = self.session_id();
            let file = file.to_string();
            tokio::spawn(async move {
                config.send_honeytoken_alert(&file, &skill, &session_id).await;
            });
        }
    }

    pub fn log_connection(&mut self, peer: &str, sni: &str) {
        self.connections += 1;
        self.tls_connections += 1;
        let line = format!("[{}] NEW CONNECTION peer={peer} sni={sni}", Self::timestamp());
        println!("{line}");
        self.write_line(&line);
    }

    pub fn log_http_hit(&mut self, peer: &str, path: &str) {
        self.connections += 1;
        self.http_hit_count += 1;
        let line = format!("[{}] HTTP HIT peer={peer} path={path}", Self::timestamp());
        println!("{line}");
        self.write_line(&line);
    }
}
