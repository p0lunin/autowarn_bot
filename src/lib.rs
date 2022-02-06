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
pub const BOT_NAME: &str = "autowarn_bot";

pub async fn setup_dispatcher(bot: Bot, db: Database) -> Dispatcher<TBot, anyhow::Error> {
    use teloxide::prelude2::*;

    let bot = bot.trace(Settings::all()).auto_send();
    let repo = WarnsRepository::new(&db);
    let storage = InMemStorage::new();
    repo.insert_default_values().await.unwrap();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(setup_warnings_handler(storage.clone()))
                .branch(setup_simple_commands()),
        )
        .branch(
            Update::filter_callback_query()
                .branch(setup_warnings_callback_queries_handler(storage.clone())),
        );

    Dispatcher::builder(bot.clone(), handler).dependencies(dptree::deps![repo, db]).build()
}
