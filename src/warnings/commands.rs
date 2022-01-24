use teloxide::utils::command::BotCommand;

#[derive(Debug, Clone, BotCommand)]
#[command(rename = "lowercase")]
pub enum WarnsCommand {
    #[command(description = "warn a user in `/warn <trigger>` format.")]
    Warn { trigger: String },
}
