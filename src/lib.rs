use shrs::prelude::*;

mod builtin;

use openai_api_rs::v1::api::Client;

pub struct OpenaiState {
    pub client: Client
}

pub struct OpenaiPlugin {
    api_key: String
}

impl OpenaiPlugin {
    pub fn new(api_key: String) -> Self {
        OpenaiPlugin {
            api_key
        }
    }
}

impl Plugin for OpenaiPlugin {
    fn init(&self, shell: &mut ShellConfig) -> anyhow::Result<()> {
        let client = Client::new(self.api_key.clone());
        let state = OpenaiState {
            client
        };
        shell.state.insert(state);
        shell.builtins.insert("ai", builtin::OpenaiBuiltin::new());
        Ok(())
    }
}
