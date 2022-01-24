use mongodb::{options::ClientOptions, Client, Database};
use roff_bot::setup_dispatcher;
use teloxide::prelude2::*;

#[tokio::main]
async fn main() {
    // TODO: maybe bot was shutdown because of network problems so we want to check
    // that we punish all user we might to, and if not so punish them.

    teloxide::enable_logging!();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
    let db = connect_to_mongo().await;

    setup_dispatcher(bot, db).await.setup_ctrlc_handler().dispatch().await;
}

async fn connect_to_mongo() -> Database {
    let env = std::env::var("MONGO_OPTIONS").expect("You must define MONGO_OPTIONS env variable.");

    log::info!("Connecting to the database...");
    let options =
        ClientOptions::parse(env).await.expect("Fail to parse MONGO_OPTIONS env variable.");
    let client = Client::with_options(options).expect("Cannot connect to the mongodb.");
    log::info!("Connected!");

    client.database("database")
}
