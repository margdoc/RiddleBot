use async_trait::async_trait;
use teloxide::{dispatching::UpdateHandler, prelude::*, utils::command::BotCommands};

use crate::{
    models,
    riddles::{self, update_data, ChatData, ChatState},
    utils::{send_message, Error, HandlerResult},
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub(crate) enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "display this text.")]
    Start,
    #[command(description = "start the riddle.")]
    StartRiddle,
    #[command(description = "stop the current riddle.")]
    StopRiddle,
}

#[derive(Clone, Default)]
enum DialogueState {
    #[default]
    None,
    StartRiddle,
    Riddle(ChatState),
}

pub(crate) fn dependencies() -> DependencyMap {
    dptree::deps![ChatData::<DialogueState>::default()]
}

pub(crate) fn schema() -> UpdateHandler<Error> {
    use dptree::case;

    Update::filter_message()
        .map_async(riddles::get_data::<DialogueState>)
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .branch(
                    case![Command::Help]
                        .inspect_async(riddles::update_data_func(DialogueState::None))
                        .endpoint(command_help),
                )
                .branch(
                    case![Command::Start]
                        .inspect_async(riddles::update_data_func(DialogueState::None))
                        .endpoint(command_help),
                )
                .branch(
                    case![Command::StartRiddle]
                        .inspect_async(riddles::update_data_func(DialogueState::StartRiddle))
                        .endpoint(command_start_riddle),
                )
                .branch(case![Command::StopRiddle].endpoint(command_stop_riddle)),
        )
        .branch(
            dptree::entry()
                .branch(case![DialogueState::StartRiddle].endpoint(command_start_riddle_code))
                .branch(case![DialogueState::Riddle(state)].endpoint(command_riddle)),
        )
}

struct Applier<'a> {
    bot: &'a Bot,
    chat_id: ChatId,
}

impl<'a> Applier<'a> {
    fn new(bot: &'a Bot, chat_id: ChatId) -> Self {
        Self { bot, chat_id }
    }
}

#[async_trait]
impl models::ActionApplier for Applier<'_> {
    async fn apply_message(&mut self, message: &str) -> HandlerResult {
        send_message(&self.bot, self.chat_id, message).await?;

        Ok(())
    }

    async fn apply_send_to(&mut self, chat_id: ChatId, message: &str) -> HandlerResult {
        send_message(&self.bot, chat_id, message).await?;

        Ok(())
    }
}

async fn command_help(bot: Bot, msg: Message) -> HandlerResult {
    send_message(&bot, msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn command_start_riddle(bot: Bot, msg: Message) -> HandlerResult {
    send_message(&bot, msg.chat.id, "What is the code of the riddle?").await?;
    Ok(())
}

async fn command_start_riddle_code(
    bot: Bot,
    msg: Message,
    riddles: riddles::Riddles,
    states: ChatData<DialogueState>,
) -> HandlerResult {
    let code = msg.text().unwrap();

    let riddles_lock = riddles.lock().await;

    let riddle_opt = riddles_lock.get(code);

    match riddle_opt {
        None => {
            send_message(&bot, msg.chat.id, "Riddle not found").await?;
        }
        Some(riddle) => {
            let chat_id = msg.chat.id;

            update_data(
                DialogueState::Riddle(ChatState {
                    riddle: code.to_owned(),
                    state: riddle.state_machine.initial_state.clone(),
                }),
                msg,
                states,
            )
            .await;

            send_message(&bot, chat_id, "Let's get started!").await?;

            send_message(
                &bot,
                chat_id,
                format!("{}\n\n{}", riddle.name, riddle.description),
            )
            .await?;
        }
    }

    Ok(())
}

async fn command_stop_riddle(
    bot: Bot,
    msg: Message,
    state: DialogueState,
    states: ChatData<DialogueState>,
) -> HandlerResult {
    match state {
        DialogueState::None => {
            send_message(&bot, msg.chat.id, "No riddle is running").await?;
        }
        DialogueState::StartRiddle => {
            send_message(&bot, msg.chat.id, "No riddle is running").await?;
        }
        DialogueState::Riddle(_) => {
            send_message(&bot, msg.chat.id, "Riddle stopped").await?;
            update_data(DialogueState::None, msg, states).await;
        }
    };
    Ok(())
}

async fn command_riddle(
    bot: Bot,
    msg: Message,
    riddles: riddles::Riddles,
    chat_state: ChatState,
    states: ChatData<DialogueState>,
) -> HandlerResult {
    let input = msg.text().unwrap();

    let riddles_lock = riddles.lock().await;
    let riddle = riddles_lock.get(&chat_state.riddle).unwrap();

    let new_state = riddle
        .state_machine
        .apply(
            &mut Applier::new(&bot, msg.chat.id),
            &chat_state.state,
            input,
        )
        .await?;

    if riddle.state_machine.is_accepting(&new_state) {
        send_message(&bot, msg.chat.id, "You solved the riddle!").await?;
        update_data(DialogueState::None, msg, states).await;
    } else {
        update_data(
            DialogueState::Riddle(ChatState {
                riddle: chat_state.riddle,
                state: new_state,
            }),
            msg,
            states,
        )
        .await;
    }

    Ok(())
}
