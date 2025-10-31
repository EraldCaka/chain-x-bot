use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct BotConfig {
    pub symbol: String,
    #[serde(rename = "backpackSymbol")]
    pub backpack_symbol: String,
    pub balance: f64,
    pub period: String,
    pub recurrence: u64,
    pub trade: f64,
}
#[derive(Debug)]
pub struct Config {
    pub api_key: String,
    pub api_secret: String,
    pub backpack_rest_url: String,
    pub backpack_ws_url: String,
    pub bot: BotConfig,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(content) = fs::read_to_string(".env") {
            for line in content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    std::env::set_var(key.trim(), value.trim());
                }
            }
        }

        let content = fs::read_to_string(path)?;
        let bot_config: BotConfig = serde_json::from_str(&content)?;
        let api_key =
            std::env::var("API_KEY").map_err(|_| "API_KEY environment variable not set")?;
        let api_secret =
            std::env::var("API_SECRET").map_err(|_| "API_SECRET environment variable not set")?;
        let backpack_rest_url = std::env::var("BACKPACK_REST_URL")
            .map_err(|_| "BACKPACK_REST_URL environment variable not set")?;
        let backpack_ws_url = std::env::var("BACKPACK_WS_URL")
            .map_err(|_| "BACKPACK_WS_URL environment variable not set")?;
        Ok(Config {
            api_key,
            api_secret,
            backpack_rest_url,
            backpack_ws_url,
            bot: bot_config,
        })
    }
}
