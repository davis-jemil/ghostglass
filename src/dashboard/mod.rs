use crate::logger::SharedLogger;
use crate::profiler::AttackerProfile;
use std::time::Duration;

const INNER: usize = 64;

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    } else {
        s.to_string()
    }
}

fn line(content: &str) -> String {
    format!("| {:<width$} |", truncate(content, INNER), width = INNER)
}

fn border() -> String {
    format!("+{}+", "-".repeat(INNER + 2))
}

fn format_uptime(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
}

/// Prints a live ASCII dashboard summarizing session activity and threat level.
pub fn print_dashboard(logger: &SharedLogger, profile: &AttackerProfile) {
    let (active_sessions, uptime) = match logger.lock() {
        Ok(log) => (log.connection_count(), log.uptime()),
        Err(_) => (0, Duration::default()),
    };

    println!("{}", border());
    println!("{}", line(&format!("{:^width$}", "GHOSTGLASS LIVE THREAT DASHBOARD", width = INNER)));
    println!("{}", border());
    println!("{}", line(&format!("Active sessions     : {active_sessions}")));
    println!("{}", line(&format!("Commands executed   : {}", profile.commands_run.len())));
    println!("{}", line(&format!("Honeytoken hits     : {}", profile.honeytoken_hits.len())));
    for hit in &profile.honeytoken_hits {
        println!("{}", line(&format!("    -> {hit}")));
    }
    println!("{}", line(&format!("Skill assessment    : {}", profile.skill_level)));
    println!("{}", line(&format!("Uptime              : {}", format_uptime(uptime))));
    println!("{}", border());
    println!("{}", line("Last 5 commands:"));
    let start = profile.commands_run.len().saturating_sub(5);
    if profile.commands_run.is_empty() {
        println!("{}", line("    (none)"));
    } else {
        for cmd in &profile.commands_run[start..] {
            println!("{}", line(&format!("    $ {cmd}")));
        }
    }
    println!("{}", border());
}
