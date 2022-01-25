mod simple_commands;
mod utils;
mod warnings;

use crate::{
    simple_commands::setup_simple_commands,
    warnings::{setup_warnings_callback_queries_handler, setup_warnings_handler, WarnsRepository},
};
use mongodb::Database;
use teloxide::{
    adaptors::{trace::Settings, Trace},
    dispatching2::{dialogue::InMemStorage, UpdateHandler},
    prelude2::*,
};

pub type TBot = AutoSend<Trace<Bot>>;
pub type HandlerOut = Result<(), anyhow::Error>;
pub type Handler = UpdateHandler<anyhow::Error>;
pub const BOT_NAME: &'static str = "autowarn_bot";

pub async fn setup_dispatcher(bot: Bot, db: Database) -> Dispatcher<TBot, anyhow::Error> {
    use teloxide::prelude2::*;

    let bot = bot.trace(Settings::all()).auto_send();
    let repo = WarnsRepository::new(&db);
    let storage = InMemStorage::new();
    repo.insert_default_values().await.unwrap();

    Dispatcher::new(bot.clone())
        .dependencies(dptree::deps![repo, db])
        .messages_handler(|h| {
            h.branch(setup_warnings_handler(storage.clone())).branch(setup_simple_commands())
        })
        .callback_queries_handler(|h| {
            h.branch(setup_warnings_callback_queries_handler(storage.clone()))
        })
}
