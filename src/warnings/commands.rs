use teloxide::utils::command::BotCommand;

#[derive(Debug, Clone, BotCommand)]
#[command(rename = "lowercase")]
pub enum WarnsCommand {
    #[command(description = "warn a user in `/warn <trigger>` format.")]
    Warn { trigger: String },
}

#[derive(Debug, Clone, BotCommand)]
#[command(rename = "lowercase")]
pub enum SetupWarnsCommands {
    #[command(
        parse_with = "split",
        description = "create new warn type for the chat with specified id."
    )]
    NewWarn { chat_id: i64 },
    #[command(description = "cancel creation of the warn.")]
    Cancel,
}
