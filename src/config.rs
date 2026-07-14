use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT_BASE_URL: &str = "https://management-api.x.ai";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Management key from console.x.ai → Settings → Management Keys.
    /// Separate from inference API keys.
    pub management_key: String,

    /// Team ID. Optional — auto-discovered via GET /auth/management-keys/validation
    /// when the key is scoped to a single team.
    #[serde(default)]
    pub team_id: Option<String>,

    /// Override base URL (testing). Defaults to https://management-api.x.ai
    #[serde(default)]
    pub base_url: Option<String>,
}

impl Config {
    pub fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .map(|s| s.trim_end_matches('/'))
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_BASE_URL)
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            PathBuf::from(home).join(".config")
        })
        .join("mcp-server-grok-management")
        .join("config.toml")
}

pub fn load() -> Result<Config> {
    let path = config_path();
    let content = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to read config file: {}\n\
             Create it with your xAI management key.\n\
             Example:\n\n\
             management_key = \"xai-...\"   # console.x.ai → Settings → Management Keys\n\
             # team_id = \"...\"           # optional — auto-discovered from key validation\n\
             # base_url = \"https://management-api.x.ai\"  # optional",
            path.display()
        )
    })?;

    let config: Config =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;

    if config.management_key.trim().is_empty() {
        bail!("management_key cannot be empty in {}", path.display());
    }

    Ok(config)
}
