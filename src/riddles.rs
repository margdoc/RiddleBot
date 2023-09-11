use std::{collections::HashMap, sync::Arc};
use teloxide::{dptree::di::Injectable, prelude::*};
use tokio::sync::Mutex;

use crate::state_machine;

#[derive(Clone, Debug)]
pub(crate) struct ChatState {
    pub riddle: String,
    pub state: String,
}

pub(crate) struct Riddle {
    pub name: String,
    pub description: String,
    pub state_machine: state_machine::StateMachine,
    pub creator: UserId,
}

pub(crate) type ChatData<D> = Arc<Mutex<HashMap<ChatId, D>>>;
pub(crate) type Riddles = Arc<Mutex<HashMap<String, Riddle>>>;

pub(crate) fn dependencies() -> DependencyMap {
    dptree::deps![Riddles::default()]
}

pub(crate) async fn get_data<D: Clone + Default>(data: ChatData<D>, msg: Message) -> D {
    data.lock()
        .await
        .get(&msg.chat.id)
        .cloned()
        .unwrap_or_default()
}

pub(crate) fn update_data_func<D: Clone + Send + Sync + std::fmt::Debug + 'static>(
    new_data: D,
) -> impl Injectable<DependencyMap, (), (Message, ChatData<D>)> {
    move |msg: Message, data_mut: ChatData<D>| {
        let new_data = new_data.clone();
        update_data(new_data, msg, data_mut)
    }
}

pub(crate) async fn update_data<D: std::fmt::Debug>(new_data: D, msg: Message, data_mut: ChatData<D>) {
    let mut data = data_mut.lock().await;
    // let prev_data = data.get(&msg.chat.id);
    // println!("chat_id: {}, prev_data: {:?}, new_data: {:?}", msg.chat.id, prev_data, new_data);
    data.insert(msg.chat.id, new_data);
}
