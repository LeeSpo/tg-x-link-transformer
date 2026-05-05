use std::sync::Arc;

use anyhow::Result;
use teloxide::{prelude::*, types::MessageId};
use tg_x_link_transformer::{
    config::{Config, LinkAction},
    link_transform::transform_links,
};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let config = Arc::new(Config::from_env()?);
    let bot = Bot::new(config.bot_token.clone());

    tracing::info!(
        "starting long polling bot with LINK_ACTION={}",
        config.link_action
    );

    teloxide::repl(bot, move |bot: Bot, message: Message| {
        let config = Arc::clone(&config);
        async move {
            if let Err(error) = handle_message(bot, message, config).await {
                tracing::warn!("failed to handle message: {error:#}");
            }
            respond(())
        }
    })
    .await;

    Ok(())
}

async fn handle_message(bot: Bot, message: Message, config: Arc<Config>) -> Result<()> {
    let Some(text) = message.text() else {
        return Ok(());
    };

    let transformed_links = transform_links(text);
    if transformed_links.is_empty() {
        return Ok(());
    }

    let response = transformed_links
        .iter()
        .map(|link| link.converted.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    match config.link_action {
        LinkAction::Reply => {
            reply_with_links(&bot, &message, response).await?;
        }
        LinkAction::DeleteResend => {
            let response = format_delete_resend_response(
                &response,
                message.from().and_then(|user| user.username.as_deref()),
            );

            if let Err(error) = bot.delete_message(message.chat.id, message.id).await {
                tracing::warn!(
                    "failed to delete original message {}; falling back to reply: {error}",
                    message.id.0
                );
                reply_with_links(&bot, &message, response).await?;
            } else {
                bot.send_message(message.chat.id, response).await?;
            }
        }
    }

    Ok(())
}

fn format_delete_resend_response(links: &str, sender_username: Option<&str>) -> String {
    match sender_username {
        Some(username) => format!("{links}\n@{username}"),
        None => links.to_string(),
    }
}

async fn reply_with_links(bot: &Bot, message: &Message, text: String) -> Result<()> {
    bot.send_message(message.chat.id, text)
        .reply_to_message_id(MessageId(message.id.0))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_resend_response_appends_sender_username() {
        let response = format_delete_resend_response(
            "https://fixupx.com/GzDTeee/status/2050185474439049466",
            Some("xxx"),
        );

        assert_eq!(
            response,
            "https://fixupx.com/GzDTeee/status/2050185474439049466\n@xxx"
        );
    }
}
