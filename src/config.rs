use std::{env, path::PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub bot_token: String,
    pub link_settings_path: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN")
            .context("TELEGRAM_BOT_TOKEN is required but was not set")?;
        let link_settings_path = env::var_os("LINK_SETTINGS_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("link-settings.json"));

        Ok(Self {
            bot_token,
            link_settings_path,
        })
    }
}
