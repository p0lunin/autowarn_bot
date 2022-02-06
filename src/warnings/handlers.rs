mod setup_warns;
mod warn;

pub use setup_warns::SetupWarnState;
use std::sync::Arc;
use teloxide::dispatching2::dialogue::InMemStorage;

use crate::{
    utils,
    warnings::{
        commands::{SetupWarnsCommands, WarnsCommand},
        repository::WarnsRepository,
    },
    HandlerOut, TBot,
};
use teloxide::prelude2::*;

type WarnsStorage = InMemStorage<SetupWarnState>;

pub fn setup_warnings_handler(storage: Arc<WarnsStorage>) -> crate::Handler {
    utils::filter_chat_owner()
        .branch(dptree::entry().filter_command::<WarnsCommand>().endpoint(handle_warns_commands))
        .branch(
            dptree::filter_map(move || {
                let storage = storage.clone();
                Some(storage)
            })
            .enter_dialogue::<Message, WarnsStorage, SetupWarnState>()
            .branch(
                dptree::entry()
                    .filter_command::<SetupWarnsCommands>()
                    .chain(dptree::filter_async(
                        |x: Dialogue<SetupWarnState, WarnsStorage>| async move {
                            if let Some(y) = x.get().await.unwrap() {
                                if matches!(y, SetupWarnState::WaitForWarnGroup(0)) {
                                    x.exit().await.unwrap();
                                }
                            }
                            true
                        },
                    ))
                    .endpoint(setup_warns::handle_setup_warns_commands),
            )
            .branch(
                dptree::entry()
                    .chain(dptree::filter_async(
                        |x: Dialogue<SetupWarnState, WarnsStorage>| async move {
                            if let Some(y) = x.get().await.unwrap() {
                                if matches!(y, SetupWarnState::WaitForWarnGroup(0)) {
                                    x.exit().await.unwrap();
                                    return false;
                                }
                            }
                            true
                        },
                    ))
                    .dispatch_by::<SetupWarnState>(),
            ),
        )
}

pub fn setup_warnings_callback_queries_handler(storage: Arc<WarnsStorage>) -> crate::Handler {
    dptree::entry().branch(
        dptree::filter_map(move || {
            let storage = storage.clone();
            Some(storage)
        })
        .enter_dialogue::<CallbackQuery, WarnsStorage, SetupWarnState>()
        .endpoint(setup_warns::wait_for_on_warn_callback_query_handler),
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
