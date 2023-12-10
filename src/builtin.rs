use shrs::prelude::*;

use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, FunctionCallType, ChatCompletionMessage, MessageRole};
use openai_api_rs::v1::common::GPT3_5_TURBO_0613;
use openai_api_rs::v1::assistant::AssistantRequest;
use openai_api_rs::v1::message::{CreateMessageRequest};
use openai_api_rs::v1::run::CreateRunRequest;
use openai_api_rs::v1::thread::CreateThreadRequest;

use crate::OpenaiState;

#[derive(Default)]
pub struct OpenaiBuiltin {

}

impl OpenaiBuiltin {
    pub fn new() -> Self {
        OpenaiBuiltin {  }
    }
}

impl BuiltinCmd for OpenaiBuiltin {
    fn run(
        &self,
        sh: &Shell,
        ctx: &mut Context,
        rt: &mut Runtime,
        args: &Vec<String>,
    ) -> anyhow::Result<CmdOutput> {

        if args.len() <= 1 {
            return Ok(CmdOutput::success());
        }

        let args = &args[1..].join(" ");

        let Some(state) = ctx.state.get_mut::<OpenaiState>() else { return Err(anyhow::anyhow!("openai state not found")) };

        // complete the prompt 
        let req = ChatCompletionRequest::new(
            GPT3_5_TURBO_0613.to_string(),
            vec![
            ChatCompletionMessage { role: MessageRole::system, content: "you will help write commands for the posix shell based on a user prompt. treat all messages after this as a request to generate a command. output only the command, do not provide any explanation".to_string(), name: None, function_call: None },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: args.to_string(),
                name: None,
                function_call: None,
            }],
        );
        // .functions(vec![chat_completion::Function {
        //     name: String::from("get_coin_price"),
        //     description: Some(String::from("Get the price of a cryptocurrency")),
        //     parameters: chat_completion::FunctionParameters {
        //         schema_type: chat_completion::JSONSchemaType::Object,
        //         properties: None,
        //         required: Some(vec![String::from("coin")]),
        //     },
        // }])
        // .function_call(FunctionCallType::Auto);

        let result = state.client.chat_completion(req)?;
        println!("{:?}", result.choices[0].message.content);
            
        Ok(CmdOutput::success())
    }
}

