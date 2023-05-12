use async_trait::async_trait;

use crate::{models_raw, utils::HandlerResult};

pub(crate) enum Prompt {
    Text(String),
    Regex(regex::Regex),
    Either,
}

impl Prompt {
    pub(crate) fn new(prompt: models_raw::Prompt) -> Self {
        match prompt {
            models_raw::Prompt::Text(text) => Prompt::Text(text),
            models_raw::Prompt::Regex(regex) => Prompt::Regex(regex::Regex::new(&regex).unwrap()),
            models_raw::Prompt::Either => Prompt::Either,
        }
    }

    pub(crate) fn matches(&self, input: &str) -> bool {
        match self {
            Prompt::Text(text) => text == input,
            Prompt::Regex(regex) => regex.is_match(input),
            Prompt::Either => true,
        }
    }
}

pub(crate) struct Edge {
    pub prompt: Prompt,
    pub actions: Vec<Action>,
    pub next: Option<String>,
}

impl Edge {
    pub(crate) fn new(edge: models_raw::Edge) -> Self {
        Self {
            prompt: Prompt::new(edge.prompt),
            actions: edge.actions.into_iter().map(Action::new).collect(),
            next: edge.next,
        }
    }
}

pub(crate) struct State {
    pub edges: Vec<Edge>,
}

impl State {
    pub(crate) fn new(state: models_raw::State) -> Self {
        Self {
            edges: state.edges.into_iter().map(Edge::new).collect(),
        }
    }
}

pub(crate) enum Action {
    Message(String),
}

#[async_trait]
pub(crate) trait ActionApplier {
    async fn apply_message(&mut self, message: &str) -> HandlerResult;
}

impl Action {
    pub(crate) fn new(action: models_raw::Action) -> Self {
        match action {
            models_raw::Action::Message(message) => Action::Message(message),
        }
    }

    pub(crate) async fn apply(&self, applier: &mut impl ActionApplier) -> HandlerResult {
        match self {
            Action::Message(message) => applier.apply_message(message).await,
        }
    }
}
