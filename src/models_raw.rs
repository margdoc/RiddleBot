use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(
    deny_unknown_fields,
    rename_all = "snake_case",
    tag = "type",
    content = "content"
)]
pub(crate) enum Prompt {
    Text(String),
    Regex(String),
    Either,
}

#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct Edge {
    pub prompt: Prompt,
    pub actions: Vec<Action>,
    pub next: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct State {
    pub name: String,
    pub edges: Vec<Edge>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(
    deny_unknown_fields,
    rename_all = "snake_case",
    tag = "type",
    content = "content"
)]
pub(crate) enum Action {
    Message(String),
    SendTo { chat_id: i64, message: String },
}

#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct StateMachine {
    pub initial_state: String,
    pub accepting_states: Vec<String>,
    pub states: Vec<State>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_state_machine() -> StateMachine {
        StateMachine {
            initial_state: "0".to_string(),
            accepting_states: vec!["2".to_string()],
            states: vec![
                State {
                    name: "0".to_string(),
                    edges: vec![
                        Edge {
                            prompt: Prompt::Text("Hello, world!".to_string()),
                            actions: vec![Action::Message("Goodbye, world!".to_string())],
                            next: Some("1".to_string()),
                        },
                        Edge {
                            prompt: Prompt::Either,
                            actions: vec![Action::Message("Nope".to_string())],
                            next: Some("2".to_string()),
                        },
                    ],
                },
                State {
                    name: "1".to_string(),
                    edges: vec![Edge {
                        prompt: Prompt::Either,
                        actions: vec![Action::Message("Nope".to_string())],
                        next: Some("0".to_string()),
                    }],
                },
                State {
                    name: "2".to_string(),
                    edges: vec![],
                },
            ],
        }
    }

    #[test]
    fn simple_state_machine_deser() {
        let state_machine_str = r#"
            {
                "initial_state": "0",
                "accepting_states": ["2"],
                "states": [
                    {
                        "name": "0",
                        "edges": [
                            {
                                "prompt": {
                                    "type": "text",
                                    "content": "Hello, world!"
                                },
                                "actions": [
                                    {
                                        "type": "message",
                                        "content": "Goodbye, world!"
                                    }
                                ],
                                "next": "1"
                            },
                            {
                                "prompt": {
                                    "type": "either"
                                },
                                "actions": [
                                    {
                                        "type": "message",
                                        "content": "Nope"
                                    }
                                ],
                                "next": "2"
                            }
                        ]
                    },
                    {
                        "name": "1",
                        "edges": [
                            {
                                "prompt": {
                                    "type": "either"
                                },
                                "actions": [
                                    {
                                        "type": "message",
                                        "content": "Nope"
                                    }
                                ],
                                "next": "0"
                            }
                        ]
                    },
                    {
                        "name": "2",
                        "edges": []
                    }
                ]
            }
        "#;

        let state_machine: StateMachine = serde_json::from_str(state_machine_str).unwrap();

        assert_eq!(state_machine, example_state_machine());
    }
}
