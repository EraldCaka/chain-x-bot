use std::env;
use std::fs;

pub struct Config {
    pub api_key: String,
    pub api_secret: String,
    pub backpack_rest_url: String,
    pub backpack_ws_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        if let Ok(content) = fs::read_to_string(".env") {
            for line in content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    env::set_var(key.trim().to_uppercase(), value.trim());
                }
            }
        }

        let api_key = env::var("API_KEY").map_err(|_| "API_KEY environment variable not set")?;
        let api_secret =
            env::var("API_SECRET").map_err(|_| "API_SECRET environment variable not set")?;
        let backpack_rest_url = env::var("BACKPACK_REST_URL")
            .map_err(|_| "BACKPACK_REST_URL environment variable not set")?;
        let backpack_ws_url = env::var("BACKPACK_WS_URL")
            .map_err(|_| "BACKPACK_WS_URL environment variable not set")?;
        Ok(Config {
            api_key,
            api_secret,
            backpack_rest_url,
            backpack_ws_url,
        })
    }
}
