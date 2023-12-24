use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use shrs::prelude::*;

use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, FunctionCallType, ChatCompletionMessage, MessageRole};
use openai_api_rs::v1::common::GPT3_5_TURBO_0613;
use openai_api_rs::v1::assistant::AssistantRequest;
use openai_api_rs::v1::message::{CreateMessageRequest};
use openai_api_rs::v1::run::CreateRunRequest;
use openai_api_rs::v1::thread::CreateThreadRequest;

use crate::{OpenaiPlugin, OpenaiState};

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
        let mut properties = HashMap::new();
        properties.insert("command".to_string(), Box::new(chat_completion::JSONSchemaDefine {
            schema_type: Some(chat_completion::JSONSchemaType::String),
            description: Some("a command to run on the command line. must be valid POSIX shell".to_string()),
            enum_values: None,
            properties: None,
            required: None,
            items: None,
        }));
        let req = ChatCompletionRequest::new(
            GPT3_5_TURBO_0613.to_string(),
            vec![
            ChatCompletionMessage { role: MessageRole::system, content: "you will help write commands for the posix shell based on a user prompt. treat all messages after this as a request to generate a command. output only the command, do not provide any explanation.".to_string(), name: None, function_call: None },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: args.to_string(),
                name: None,
                function_call: None,
            }],
        )
        .functions(vec![chat_completion::Function {
            name: String::from("shell_command"),
            description: Some(String::from("a command to run on the command line")),
            parameters: chat_completion::FunctionParameters {
                schema_type: chat_completion::JSONSchemaType::Object,
                properties: Some(properties),
                required: Some(vec![String::from("command")]),
            },
        }])
        .function_call(FunctionCallType::Auto);

        let result = state.client.chat_completion(req)?;
        match result.choices[0].finish_reason {
            Some(chat_completion::FinishReason::function_call) => {

                #[derive(Debug, Serialize, Deserialize)]
                struct Command {
                    command: String,
                }

                let function_call = result.choices[0].message.function_call.as_ref().unwrap();
                let fn_name = function_call.name.clone().unwrap();
                let arguments = function_call.arguments.clone().unwrap();
                if fn_name == "shell_command" {
                    let cmd: Command = serde_json::from_str(&arguments)?;
                    println!("{}", cmd.command);
                } else {
                    eprintln!("unhandled function call: {fn_name}");
                }
            },
            _ => {
                eprintln!("Unable to generate command");
            }
        }
            
        Ok(CmdOutput::success())
    }
}

