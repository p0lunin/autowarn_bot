mod utils;
mod warnings;

use crate::warnings::{setup_warnings_handler, WarnsRepository};
use mongodb::Database;
use teloxide::{
    adaptors::{trace::Settings, Trace},
    dispatching2::UpdateHandler,
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
    repo.insert_default_values().await.unwrap();

    Dispatcher::new(bot.clone())
        .dependencies(dptree::deps![repo, db])
        .messages_handler(|h| h.branch(setup_warnings_handler()))
}
