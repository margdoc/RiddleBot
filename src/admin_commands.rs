use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashSet;
use teloxide::dispatching::UpdateHandler;
use teloxide::types::ParseMode;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::riddles::{self, ChatData};
use crate::utils::{escape_chars, send_message, Error, HandlerResult};
use crate::{commands, models_raw, state_machine};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These admin commands are supported:"
)]
enum AdminCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "create a new riddle.")]
    NewRiddle,
    #[command(description = "remove a riddle.")]
    RemoveRiddle,
    #[command(description = "list all riddles.")]
    ListRiddles,
}

#[derive(Clone, Debug)]
struct RiddleName(String);
#[derive(Clone, Debug)]
struct RiddleDescription(String);

#[derive(Clone, Debug)]
enum NewRiddleState {
    Code,
    Name {
        code: Option<String>,
    },
    Description {
        code: Option<String>,
        name: RiddleName,
    },
    StateMachine {
        code: Option<String>,
        name: RiddleName,
        description: RiddleDescription,
    },
}

#[derive(Clone, Debug)]
enum RemoveRiddleState {
    Code,
}

#[derive(Clone, Default, Debug)]
enum DialogueState {
    #[default]
    None,
    NewRiddle(NewRiddleState),
    RemoveRiddle(RemoveRiddleState),
}

fn admins_from_env() -> HashSet<UserId> {
    option_env!("ADMINS")
        .unwrap_or("")
        .split(',')
        .flat_map(|s| s.parse::<u64>())
        .map(UserId)
        .collect::<HashSet<_>>()
}

pub(crate) fn dependencies() -> DependencyMap {
    dptree::deps![ChatData::<DialogueState>::default()]
}

pub(crate) fn schema() -> UpdateHandler<Error> {
    use dptree::case;

    let admins = admins_from_env();

    let is_admin = move |msg: Message| admins.contains(&msg.from().as_ref().unwrap().id);

    Update::filter_message()
        .filter(is_admin)
        .map_async(riddles::get_data::<DialogueState>)
        .branch(
            dptree::entry()
                .filter_command::<AdminCommand>()
                .branch(
                    case![AdminCommand::Help]
                        .inspect_async(riddles::update_data_func(DialogueState::None))
                        .endpoint(command_help),
                )
                .branch(
                    case![AdminCommand::ListRiddles]
                        .inspect_async(riddles::update_data_func(DialogueState::None))
                        .endpoint(command_list_riddles),
                )
                .branch(
                    case![AdminCommand::NewRiddle]
                        .inspect_async(riddles::update_data_func(DialogueState::NewRiddle(
                            NewRiddleState::Code,
                        )))
                        .endpoint(command_new_riddle),
                )
                .branch(
                    case![AdminCommand::RemoveRiddle]
                        .inspect_async(riddles::update_data_func(DialogueState::RemoveRiddle(
                            RemoveRiddleState::Code,
                        )))
                        .endpoint(command_remove_riddle),
                ),
        )
        .branch(
            dptree::entry()
                .branch(
                    case![DialogueState::NewRiddle(new_riddle_state)]
                        .branch(case![NewRiddleState::Code].endpoint(new_riddle_code))
                        .branch(case![NewRiddleState::Name { code }].endpoint(new_riddle_name))
                        .branch(
                            case![NewRiddleState::Description { code, name }]
                                .endpoint(new_riddle_description),
                        )
                        .branch(
                            case![NewRiddleState::StateMachine {
                                code,
                                name,
                                description
                            }]
                            .endpoint(new_riddle_state_machine),
                        ),
                )
                .branch(
                    case![DialogueState::RemoveRiddle(remove_riddle_state)]
                        .branch(case![RemoveRiddleState::Code])
                        .endpoint(remove_riddle_code),
                ),
        )
}

async fn command_help(bot: Bot, msg: Message) -> HandlerResult {
    send_message(
        &bot,
        msg.chat.id,
        commands::Command::descriptions().to_string(),
    )
    .await?;
    send_message(&bot, msg.chat.id, AdminCommand::descriptions().to_string()).await?;
    Ok(())
}

async fn command_list_riddles(bot: Bot, riddles: riddles::Riddles, msg: Message) -> HandlerResult {
    let riddles = riddles.lock().await;
    let riddles_string = riddles
        .iter()
        .map(|(code, riddle)| {
            format!(
                "{} \\(code: `{}`\\)\n[Author](tg://user?id={})\nDescription:\n{}",
                escape_chars(riddle.name.clone()),
                code,
                riddle.creator,
                escape_chars(riddle.description.clone())
            )
        })
        .fold("List of riddles:".to_owned(), |acc, s| acc + "\n\n" + &s);

    bot.send_message(msg.chat.id, riddles_string)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

static RANDOM_RIDDLE_CODE: &str = "RANDOM";

async fn command_new_riddle(bot: Bot, msg: Message) -> HandlerResult {
    send_message(
        &bot,
        msg.chat.id,
        format!(
            "What is the code for your riddle? ({} if you want us to randomize)",
            RANDOM_RIDDLE_CODE
        ),
    )
    .await?;
    Ok(())
}

async fn command_remove_riddle(bot: Bot, msg: Message) -> HandlerResult {
    send_message(&bot, msg.chat.id, "What is the code of the riddle?").await?;
    Ok(())
}

async fn new_riddle_code(
    bot: Bot,
    msg: Message,
    dialogue_state_mut: ChatData<DialogueState>,
    riddles_mut: riddles::Riddles,
) -> HandlerResult {
    let code = msg.text().unwrap();
    let chat_id = msg.chat.id;
    let riddles = riddles_mut.lock().await;
    if riddles.contains_key(code) {
        send_message(
            &bot,
            chat_id,
            format!("Riddle with code `{}` already exists!", code),
        )
        .await?;
        return Ok(());
    }
    riddles::update_data(
        DialogueState::NewRiddle(NewRiddleState::Name {
            code: if code == RANDOM_RIDDLE_CODE {
                None
            } else {
                Some(code.to_owned())
            },
        }),
        msg,
        dialogue_state_mut,
    )
    .await;
    send_message(&bot, chat_id, "What is the name of the riddle?").await?;
    Ok(())
}

async fn new_riddle_name(
    bot: Bot,
    msg: Message,
    dialogue_state_mut: ChatData<DialogueState>,
    code: Option<String>,
) -> HandlerResult {
    let name = msg.text().unwrap();
    send_message(&bot, msg.chat.id, "What is the description of the riddle?").await?;
    riddles::update_data(
        DialogueState::NewRiddle(NewRiddleState::Description {
            code,
            name: RiddleName(name.to_owned()),
        }),
        msg,
        dialogue_state_mut,
    )
    .await;
    Ok(())
}

async fn new_riddle_description(
    bot: Bot,
    msg: Message,
    dialogue_state_mut: ChatData<DialogueState>,
    (code, name): (Option<String>, RiddleName),
) -> HandlerResult {
    let description = msg.text().unwrap();
    send_message(
        &bot,
        msg.chat.id,
        "What is the state machine code of the riddle?",
    )
    .await?;
    riddles::update_data(
        DialogueState::NewRiddle(NewRiddleState::StateMachine {
            code,
            name,
            description: RiddleDescription(description.to_owned()),
        }),
        msg,
        dialogue_state_mut,
    )
    .await;
    Ok(())
}

fn random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

async fn new_riddle_state_machine(
    bot: Bot,
    msg: Message,
    dialogue_state_mut: ChatData<DialogueState>,
    (code, name, description): (Option<String>, RiddleName, RiddleDescription),
    riddles_mut: riddles::Riddles,
) -> HandlerResult {
    let state_machine_str = msg.text().unwrap();

    match serde_json::from_str::<models_raw::StateMachine>(state_machine_str) {
        Ok(state_machine) => {
            let chat_id = msg.chat.id;

            riddles::update_data(DialogueState::None, msg.clone(), dialogue_state_mut).await;

            let riddle = riddles::Riddle {
                name: name.0,
                description: description.0,
                creator: msg.from().unwrap().id,
                state_machine: state_machine::StateMachine::new(state_machine),
            };

            let code = match code {
                Some(code) => {
                    let mut riddles = riddles_mut.lock().await;
                    if riddles.contains_key(&code) {
                        send_message(
                            &bot,
                            chat_id,
                            format!("Riddle with code `{}` already exists!", code),
                        )
                        .await?;
                        return Ok(());
                    }

                    riddles.insert(code.clone(), riddle);
                    code
                }
                None => {
                    let mut riddles = riddles_mut.lock().await;
                    let code = loop {
                        let code = random_string();
                        if !riddles.contains_key(&code) {
                            break code;
                        }
                    };

                    riddles.insert(code.clone(), riddle);
                    code
                }
            };

            send_message(&bot, chat_id, format!("Riddle created\n! Code: `{}`", code)).await?;
        }
        Err(e) => {
            send_message(&bot, msg.chat.id, format!("Error: {}", e)).await?;
        }
    }

    Ok(())
}

async fn remove_riddle_code(
    bot: Bot,
    msg: Message,
    dialogue_state_mut: ChatData<DialogueState>,
    riddles_mut: riddles::Riddles,
) -> HandlerResult {
    let code = msg.text().unwrap();

    let mut riddles = riddles_mut.lock().await;
    if riddles.remove(code).is_some() {
        riddles::update_data(DialogueState::None, msg.clone(), dialogue_state_mut).await;
        send_message(&bot, msg.chat.id, "Riddle removed!").await?;
    } else {
        send_message(&bot, msg.chat.id, "Riddle not found!").await?;
    }

    Ok(())
}
