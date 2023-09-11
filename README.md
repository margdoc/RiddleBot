# RiddleBot

## Setup
Set enviroment variables:
* `TELOXIDE_TOKEN` - a token of your bot (from [BotFather](https://t.me/botfather))
* `ADMINS` - comma-separated list of admins' ids

## Available commands
For admins:
* `/help`
* `/newriddle` starts the dialogue where it expects the riddle's code, name, description and state machine description
* `/removeriddle` starts the dialogue where it expects the riddle's code
* `/listriddles`

For users:
* `/help`
* `/startriddle` starts the dialogue where it expects the riddle's code
* `/stopriddle`

## Riddles
Riddles are associated with a code - randomly generated string. It is returned to the user when he creates a new riddle.

## State machine
For now, state machines can be only created from a description in the JSON format. Schema:

```
StateMachine {
    initial_state: string,
    accepting_states: [string],
    states: [State]
}

State {
    name: string,
    edges: [Edge]
}

Edge {
    // a rule that if the user's input satisfies, the state machine will
    // transition along this edge
    prompt: Prompt,
    // actions to be executed before switching to the new state
    actions: [Action],
    // next state
    next: string,
}

Prompt {
    type: "text",
    content: string,
} |  {
    type: "regex",
    content: string,
} | {
    type: "either"
}

Action {
    type: "message",
    content: string
} | {
    type: "send_to",
    content: {
        chat_id: int,
        message: string
    }
}
```
Example:
```
{
    "initial_state": "start",
    "accepting_states": ["end"],
    "states": [
        {
            "name": "start",
            "edges": [
                {
                    "prompt": {
                        "type": "text",
                        "content": "start"
                    },
                    "actions": [
                        {
                            "type": "message",
                            "content": "Hello!"
                        }
                    ],
                    "next": "end"
                }
            ]
        },
        {
            "name": "end",
            "edges": []
        }
    ]
}
```
This state machine will accept only one input - "start" and will respond with "Hello!". After that, it will stop.