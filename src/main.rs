use teloxide::prelude::*;

mod admin_commands;
mod commands;
mod models;
mod models_raw;
mod riddles;
mod state_machine;
mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(admin_commands::schema())
        .branch(commands::schema());

    let mut dependencies = riddles::dependencies();
    dependencies.insert_container(admin_commands::dependencies());
    dependencies.insert_container(commands::dependencies());

    Dispatcher::builder(bot, handler)
        .dependencies(dependencies)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
