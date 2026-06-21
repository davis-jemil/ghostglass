const CONFIG_PATH: &str = "config/alerts.toml";

#[derive(Debug, Clone, Default)]
pub struct AlertConfig {
    pub webhook_url: Option<String>,
    pub email_to: Option<String>,
    pub enabled: bool,
}

impl AlertConfig {
    /// Loads `config/alerts.toml`. Returns `None` if the file doesn't exist,
    /// so callers can silently disable alerts rather than erroring out.
    pub fn load() -> Option<Self> {
        let content = std::fs::read_to_string(CONFIG_PATH).ok()?;
        Some(Self::parse(&content))
    }

    fn parse(content: &str) -> Self {
        let mut config = Self::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            match key {
                "enabled" => config.enabled = value.eq_ignore_ascii_case("true"),
                "webhook_url" => {
                    config.webhook_url = (!value.is_empty()).then(|| value.to_string());
                }
                "email_to" => {
                    config.email_to = (!value.is_empty()).then(|| value.to_string());
                }
                _ => {}
            }
        }

        config
    }

    /// Fires a honeytoken alert. No-op unless alerts are enabled and a webhook is set.
    pub async fn send_honeytoken_alert(&self, file: &str, skill: &str, session_id: &str) {
        let Some(url) = self.target_url() else { return };
        let content = format!(
            "🚨 GHOSTGLASS ALERT\nHoneytoken triggered: {file}\nAttacker skill: {skill}\nSession: {session_id}"
        );
        post_webhook(url, content).await;
    }

    /// Fires when an attacker's skill assessment reaches APT level.
    pub async fn send_apt_alert(&self, session_id: &str) {
        let Some(url) = self.target_url() else { return };
        let content = format!("⚠️ APT-LEVEL THREAT DETECTED\nSession: {session_id}");
        post_webhook(url, content).await;
    }

    fn target_url(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        self.webhook_url.clone()
    }
}

async fn post_webhook(url: String, content: String) {
    let payload = serde_json::json!({ "content": content });
    let client = reqwest::Client::new();
    match client.post(&url).json(&payload).send().await {
        Ok(resp) if !resp.status().is_success() => {
            eprintln!("[alerts] webhook responded with status {}", resp.status());
        }
        Err(e) => eprintln!("[alerts] webhook send failed: {e}"),
        _ => {}
    }
}
