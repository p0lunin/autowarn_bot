mod warn;

use crate::{
    utils,
    warnings::{commands::WarnsCommand, repository::WarnsRepository},
    HandlerOut, TBot, BOT_NAME,
};
use teloxide::prelude2::*;

pub fn setup_warnings_handler() -> crate::Handler {
    dptree::entry().branch(
        utils::filter_chat_owner()
            .add_command::<WarnsCommand>(BOT_NAME.into())
            .endpoint(handle_warns_commands),
    )
}

async fn handle_warns_commands(
    bot: TBot,
    mes: Message,
    cmd: WarnsCommand,
    repo: WarnsRepository,
) -> HandlerOut {
    match cmd {
        WarnsCommand::Warn { trigger } => {
            let reply_to_message = match mes.reply_to_message() {
                Some(mes) => mes.clone(),
                None => {
                    bot.send_message(mes.chat.id, "Reply to a user message to warn.").await?;
                    return Ok(());
                }
            };
            let reply_to = match reply_to_message.from() {
                Some(user) => user.clone(),
                None => {
                    bot.send_message(mes.chat.id, "Reply to a user message to warn.").await?;
                    return Ok(());
                }
            };
            let warn_info = repo.find_warn_by_trigger(trigger.as_str()).await?;
            match warn_info {
                Some(warn) => {
                    warn::warn_user(bot.clone(), mes, repo, reply_to, &warn).await?;
                    warn::on_warn(bot, &reply_to_message, warn.on_warn).await?;
                }
                None => {
                    bot.send_message(mes.chat.id, "There are no such warning type.").await?;
                }
            }
        }
    }

    Ok(())
}
