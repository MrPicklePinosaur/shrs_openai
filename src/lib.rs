use shrs::prelude::*;

mod builtin;

use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, FunctionCallType, ChatCompletionMessage, MessageRole};

pub struct OpenaiState {
    pub client: Client,
    pub chat_history: Vec<ChatCompletionMessage>,
}

pub struct OpenaiPlugin {
    api_key: String,
}

impl OpenaiPlugin {
    pub fn new(api_key: String) -> Self {
        OpenaiPlugin {
            api_key,
        }
    }
}

impl Plugin for OpenaiPlugin {
    fn init(&self, shell: &mut ShellConfig) -> anyhow::Result<()> {

        // Provide system message
        let chat_history = vec![
            ChatCompletionMessage { role: MessageRole::system, content: "you will help write commands for the posix shell based on a user prompt. treat all messages after this as a request to generate a command. output only the command, do not provide any explanation.".to_string(), name: None, function_call: None },
        ];

        let client = Client::new(self.api_key.clone());
        let state = OpenaiState {
            client,
            chat_history
        };

        shell.state.insert(state);
        shell.builtins.insert("ai", builtin::OpenaiBuiltin::new());
        Ok(())
    }
}
