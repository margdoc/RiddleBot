use std::collections::HashMap;
use std::collections::HashSet;

use crate::models;
use crate::models_raw;
use crate::utils::HandlerResult;

pub(crate) struct StateMachine {
    pub initial_state: String,
    accepting_states: HashSet<String>,
    states: HashMap<String, models::State>,
}

// #[derive(thiserror::Error)]
// enum StateMachineError {

// }

impl StateMachine {
    pub(crate) fn new(state_machine_raw: models_raw::StateMachine) -> Self {
        let mut states = HashMap::new();
        for state in state_machine_raw.states {
            states.insert(state.name.clone(), models::State::new(state));
        }

        Self {
            initial_state: state_machine_raw.initial_state,
            accepting_states: state_machine_raw.accepting_states.into_iter().collect(),
            states,
        }
    }

    pub(crate) async fn apply(
        &self,
        applier: &mut impl models::ActionApplier,
        state_name: &str,
        input: &str,
    ) -> HandlerResult<String> {
        let state = self.states.get(state_name).unwrap();
        let edge_opt = state.edges.iter().find(|edge| edge.prompt.matches(input));

        match edge_opt {
            None => Ok(state_name.to_string()),
            Some(edge) => {
                for action in &edge.actions {
                    action.apply(applier).await?;
                }

                if let Some(next) = edge.next.as_ref() {
                    Ok(next.clone())
                } else {
                    Ok(state_name.to_string())
                }
            }
        }
    }

    pub(crate) fn is_accepting(&self, state_name: &str) -> bool {
        self.accepting_states.contains(state_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;

    struct Applier {
        messages: Vec<String>,
    }

    impl Applier {
        fn new() -> Self {
            Self { messages: vec![] }
        }
    }

    #[async_trait]
    impl super::models::ActionApplier for Applier {
        async fn apply_message(&mut self, message: &str) -> super::HandlerResult {
            self.messages.push(message.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn simple_test() {
        let state_machine = StateMachine {
            initial_state: "1".to_string(),
            accepting_states: vec!["3".to_string()].into_iter().collect(),
            states: HashMap::from([
                (
                    "1".to_string(),
                    models::State {
                        edges: vec![models::Edge {
                            prompt: models::Prompt::Text("1-2".to_string()),
                            next: Some("2".to_string()),
                            actions: vec![models::Action::Message("1-2".to_string())],
                        }],
                    },
                ),
                (
                    "2".to_string(),
                    models::State {
                        edges: vec![
                            models::Edge {
                                prompt: models::Prompt::Text("2-1".to_string()),
                                next: Some("1".to_string()),
                                actions: vec![models::Action::Message("2-1".to_string())],
                            },
                            models::Edge {
                                prompt: models::Prompt::Either,
                                next: Some("3".to_string()),
                                actions: vec![models::Action::Message("2-3".to_string())],
                            },
                        ],
                    },
                ),
            ]),
        };

        let mut applier = Applier::new();
        let mut state_name = state_machine.initial_state.clone();
        state_name = state_machine
            .apply(&mut applier, &state_name, "1-2")
            .await
            .unwrap();
        state_name = state_machine
            .apply(&mut applier, &state_name, "blep")
            .await
            .unwrap();
        assert_eq!(state_name, "3");
        assert_eq!(applier.messages, vec!["1-2".to_string(), "2-3".to_string()]);
        assert!(state_machine.is_accepting(&state_name));

        applier = Applier::new();
        state_name = state_machine.initial_state.clone();
        state_name = state_machine
            .apply(&mut applier, &state_name, "1-2")
            .await
            .unwrap();
        state_name = state_machine
            .apply(&mut applier, &state_name, "2-1")
            .await
            .unwrap();
        state_name = state_machine
            .apply(&mut applier, &state_name, "1-2")
            .await
            .unwrap();
        state_name = state_machine
            .apply(&mut applier, &state_name, "nghu")
            .await
            .unwrap();
        assert_eq!(state_name, "3");
        assert_eq!(
            applier.messages,
            vec![
                "1-2".to_string(),
                "2-1".to_string(),
                "1-2".to_string(),
                "2-3".to_string()
            ]
        );
        assert!(state_machine.is_accepting(&state_name));

        applier = Applier::new();
        state_name = state_machine.initial_state.clone();
        state_name = state_machine
            .apply(&mut applier, &state_name, "1-2")
            .await
            .unwrap();
        state_name = state_machine
            .apply(&mut applier, &state_name, "2-1")
            .await
            .unwrap();
        assert_eq!(state_name, "1");
        assert_eq!(applier.messages, vec!["1-2".to_string(), "2-1".to_string()]);
        assert!(!state_machine.is_accepting(&state_name));
    }
}
