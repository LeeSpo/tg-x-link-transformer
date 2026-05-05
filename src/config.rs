use std::{env, fmt, str::FromStr};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub bot_token: String,
    pub link_action: LinkAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkAction {
    Reply,
    DeleteResend,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN")
            .context("TELEGRAM_BOT_TOKEN is required but was not set")?;
        let link_action = env::var("LINK_ACTION")
            .unwrap_or_else(|_| LinkAction::Reply.to_string())
            .parse()?;

        Ok(Self {
            bot_token,
            link_action,
        })
    }
}

impl FromStr for LinkAction {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "reply" => Ok(Self::Reply),
            "delete_resend" => Ok(Self::DeleteResend),
            other => {
                bail!("unsupported LINK_ACTION `{other}`; expected `reply` or `delete_resend`")
            }
        }
    }
}

impl fmt::Display for LinkAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reply => formatter.write_str("reply"),
            Self::DeleteResend => formatter.write_str("delete_resend"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reply_action() {
        assert_eq!("reply".parse::<LinkAction>().unwrap(), LinkAction::Reply);
    }

    #[test]
    fn parses_delete_resend_action() {
        assert_eq!(
            "delete_resend".parse::<LinkAction>().unwrap(),
            LinkAction::DeleteResend
        );
    }

    #[test]
    fn rejects_unknown_action() {
        let error = "delete".parse::<LinkAction>().unwrap_err().to_string();

        assert!(error.contains("unsupported LINK_ACTION"));
    }
}
