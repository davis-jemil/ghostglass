use std::collections::HashSet;
use std::fmt;
use std::path::Path;

const COMMAND_MARKER: &str = "] COMMAND: ";
const HONEYTOKEN_MARKER: &str = "] HONEYTOKEN TRIGGERED: ";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillLevel {
    ScriptKiddie,
    Intermediate,
    Advanced,
    Apt,
}

impl fmt::Display for SkillLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            SkillLevel::ScriptKiddie => "Script Kiddie",
            SkillLevel::Intermediate => "Intermediate",
            SkillLevel::Advanced => "Advanced",
            SkillLevel::Apt => "APT",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone)]
pub struct AttackerProfile {
    pub session_id: String,
    pub commands_run: Vec<String>,
    pub honeytoken_hits: Vec<String>,
    pub skill_level: SkillLevel,
}

fn command_target(cmd: &str) -> Option<&str> {
    cmd.split_whitespace().skip(1).find(|a| !a.starts_with('-'))
}

/// Scores observed behavior against the four threat tiers. "Across sessions"
/// is approximated within a single log by combining honeytoken breadth with
/// evidence of systematic traversal, since cross-session correlation isn't
/// tracked yet.
pub fn assess_skill(commands: &[String], honeytoken_hits: &[String]) -> SkillLevel {
    let lower: Vec<String> = commands.iter().map(|c| c.to_lowercase()).collect();

    let priv_esc = lower.iter().any(|c| {
        let first = c.split_whitespace().next().unwrap_or("");
        first == "sudo" || first == "su" || c.contains("/etc/shadow") || c.contains("/etc/passwd")
    });

    let network_probe = lower.iter().any(|c| {
        let mut parts = c.split_whitespace();
        match parts.next().unwrap_or("") {
            "ifconfig" => true,
            "ip" => matches!(parts.next(), Some("a") | Some("addr") | Some("address")),
            _ => false,
        }
    });

    let key_reads = lower.iter().any(|c| c.contains(".env") || c.contains(".key"));

    let traversal_targets: HashSet<&str> = lower
        .iter()
        .filter(|c| matches!(c.split_whitespace().next(), Some("ls") | Some("cd")))
        .filter_map(|c| command_target(c))
        .collect();

    let systematic_traversal = traversal_targets.len() >= 3;
    let multi_honeytoken = honeytoken_hits.len() >= 2;

    if systematic_traversal && multi_honeytoken {
        SkillLevel::Apt
    } else if network_probe || key_reads {
        SkillLevel::Advanced
    } else if priv_esc {
        SkillLevel::Intermediate
    } else {
        SkillLevel::ScriptKiddie
    }
}

fn session_id_from_path(log_file: &str) -> String {
    Path::new(log_file)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| log_file.to_string())
}

/// Reads a session log written by `SessionLogger` and scores the attacker's skill.
pub fn profile_session(log_file: &str) -> AttackerProfile {
    let content = std::fs::read_to_string(log_file).unwrap_or_default();
    let mut commands_run = Vec::new();
    let mut honeytoken_hits = Vec::new();

    for line in content.lines() {
        if let Some(idx) = line.find(HONEYTOKEN_MARKER) {
            honeytoken_hits.push(line[idx + HONEYTOKEN_MARKER.len()..].trim().to_string());
        } else if let Some(idx) = line.find(COMMAND_MARKER) {
            commands_run.push(line[idx + COMMAND_MARKER.len()..].trim().to_string());
        }
    }

    let skill_level = assess_skill(&commands_run, &honeytoken_hits);

    AttackerProfile {
        session_id: session_id_from_path(log_file),
        commands_run,
        honeytoken_hits,
        skill_level,
    }
}

/// Prints a formatted threat assessment report for the given profile.
pub fn print_profile(profile: &AttackerProfile) {
    println!("==================== THREAT ASSESSMENT ====================");
    println!(" Session ID       : {}", profile.session_id);
    println!(" Commands Run     : {}", profile.commands_run.len());
    println!(" Honeytoken Hits  : {}", profile.honeytoken_hits.len());
    for hit in &profile.honeytoken_hits {
        println!("   - {hit}");
    }
    println!(" Skill Assessment : {}", profile.skill_level);
    println!("=============================================================");
}
