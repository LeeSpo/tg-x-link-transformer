use teloxide::types::BotCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkDeleteCommand {
    Set(bool),
    Invalid,
}

pub fn bot_commands() -> Vec<BotCommand> {
    vec![BotCommand::new(
        "link_delete",
        "Set link delete mode for this chat",
    )]
}

pub fn parse_link_delete_command(text: &str) -> Option<LinkDeleteCommand> {
    let mut parts = text.split_whitespace();
    let command = parts.next()?;
    let command = command.strip_prefix('/')?;
    let command = command
        .split_once('@')
        .map_or(command, |(command, _bot_name)| command);

    if !command.eq_ignore_ascii_case("link_delete") {
        return None;
    }

    let Some(value) = parts.next() else {
        return Some(LinkDeleteCommand::Invalid);
    };

    if parts.next().is_some() {
        return Some(LinkDeleteCommand::Invalid);
    }

    match value.to_ascii_lowercase().as_str() {
        "true" => Some(LinkDeleteCommand::Set(true)),
        "false" => Some(LinkDeleteCommand::Set(false)),
        _ => Some(LinkDeleteCommand::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_true_value_case_insensitively() {
        assert_eq!(
            parse_link_delete_command("/link_delete TrUe"),
            Some(LinkDeleteCommand::Set(true))
        );
    }

    #[test]
    fn accepts_false_value_case_insensitively() {
        assert_eq!(
            parse_link_delete_command("/LINK_DELETE false"),
            Some(LinkDeleteCommand::Set(false))
        );
    }

    #[test]
    fn accepts_bot_mention_suffix() {
        assert_eq!(
            parse_link_delete_command("/LINK_DELETE@my_bot TRUE"),
            Some(LinkDeleteCommand::Set(true))
        );
    }

    #[test]
    fn rejects_missing_value() {
        assert_eq!(
            parse_link_delete_command("/LINK_DELETE"),
            Some(LinkDeleteCommand::Invalid)
        );
    }

    #[test]
    fn rejects_invalid_value() {
        assert_eq!(
            parse_link_delete_command("/LINK_DELETE maybe"),
            Some(LinkDeleteCommand::Invalid)
        );
    }

    #[test]
    fn ignores_unrelated_messages() {
        assert_eq!(parse_link_delete_command("hello /LINK_DELETE TRUE"), None);
        assert_eq!(parse_link_delete_command("/start"), None);
    }

    #[test]
    fn bot_commands_registers_link_delete_completion() {
        let commands = bot_commands();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].command, "link_delete");
        assert_eq!(
            commands[0].description,
            "Set link delete mode for this chat"
        );
    }
}
