use crate::{dptree, Message, TBot, BOT_NAME};
use teloxide::{prelude2::*, types::ParseMode, utils::command::BotCommand};

#[derive(Debug, Clone, BotCommand)]
#[command(rename = "lowercase")]
pub enum SimpleCommands {
    #[command(description = "shows this chat ID.")]
    ChatId,
    #[command(description = "shows your ID.")]
    MyId,
}

pub fn setup_simple_commands() -> crate::Handler {
    dptree::entry().add_command::<SimpleCommands>(BOT_NAME.into()).branch(dptree::endpoint(
        |bot: TBot, mes: Message, cmd: SimpleCommands| async move {
            let text = match cmd {
                SimpleCommands::MyId => {
                    format!("Your ID: `{}`", mes.from().unwrap().id)
                }
                SimpleCommands::ChatId => {
                    format!("Chat ID: `{}`", mes.chat.id)
                }
            };
            bot.send_message(mes.chat.id, text)
                .reply_to_message_id(mes.id)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;

            Ok(())
        },
    ))
}
