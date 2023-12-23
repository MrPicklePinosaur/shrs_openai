use shrs::prelude::*;

mod builtin;

use openai_api_rs::v1::api::Client;

pub struct OpenaiPlugin {
    api_key: String,
    client: Client
}

impl OpenaiPlugin {
    pub fn new(api_key: String) -> Self {
        let client = Client::new(api_key.clone());
        OpenaiPlugin {
            api_key,
            client
        }
    }
}

impl Plugin for OpenaiPlugin {
    fn init(&self, shell: &mut ShellConfig) -> anyhow::Result<()> {
        shell.builtins.insert("ai", builtin::OpenaiBuiltin::new());
        Ok(())
    }
}
