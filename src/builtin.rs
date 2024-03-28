use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use shrs::prelude::*;
use openai_api_rs::v1::api::Client;

use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, ChatCompletionMessage, MessageRole, Content};
use openai_api_rs::v1::common::GPT3_5_TURBO_0613;
use openai_api_rs::v1::assistant::AssistantRequest;
use openai_api_rs::v1::message::{CreateMessageRequest};
use openai_api_rs::v1::run::CreateRunRequest;
use openai_api_rs::v1::thread::CreateThreadRequest;

use crate::{OpenaiPlugin, OpenaiState};

#[derive(Debug, Serialize, Deserialize)]
struct Command {
    command: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Explanation {
    plaintext: String,
}

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
        args: &[String],
    ) -> anyhow::Result<CmdOutput> {

        if args.len() <= 1 {
            return Ok(CmdOutput::success());
        }

        // parse flags
        let mut flag_explain = false;
        let mut shift = 1;

        if args.len() >= 2 {
            if args[1] == "-e" {
                println!("passed explain flag");
                flag_explain = true;
                shift += 1;
            }
        }

        let args = &args[shift..].join(" ");
        println!("args {args}");

        let Some(state) = ctx.state.get_mut::<OpenaiState>() else { return Err(anyhow::anyhow!("openai state not found")) };

        state.chat_history.push(
            chat_completion::ChatCompletionMessage { role: chat_completion::MessageRole::user,
                content: Content::Text(args.to_string()),
                name: None,
            }
        );

        // Use different supported function calls depending on explanation mode or command mode. We
        // however do use the same history to share context
        let req = if flag_explain {

            ChatCompletionRequest::new(
                GPT3_5_TURBO_0613.to_string(),
                state.chat_history.clone() // TODO this can be a pretty big copy
            )
            .tools(self.get_explain_completions())
            .tool_choice(chat_completion::ToolChoiceType::Auto)

        } else {

            ChatCompletionRequest::new(
                GPT3_5_TURBO_0613.to_string(),
                state.chat_history.clone() // TODO this can be a pretty big copy
            )
            .tools(self.get_command_completions())
            .tool_choice(chat_completion::ToolChoiceType::Auto)
        };

        let result = state.client.chat_completion(req)?;
        match result.choices[0].finish_reason {
            Some(chat_completion::FinishReason::tool_calls) => {


                let tool_calls = result.choices[0].message.tool_calls.as_ref().unwrap();
                for tool_call in tool_calls {
                    let fn_name = tool_call.function.name.clone().unwrap();
                    let arguments = tool_call.function.arguments.clone().unwrap();
                    if fn_name == "shell_command" {
                        let cmd: Command = serde_json::from_str(&arguments)?;
                        // TODO could make auto-run configurable
                        ctx.prompt_content_queue.push(PromptContent { content: cmd.command, auto_run: false });
                    } else if fn_name == "explanation" {
                        let explanation: Explanation = serde_json::from_str(&arguments)?;
                        println!("{}", explanation.plaintext);
                    } else {
                        eprintln!("unhandled function call: {fn_name}");
                    }
                }
            },
            _ => {
                eprintln!("Unable to generate command");
            }
        }
            
        Ok(CmdOutput::success())
    }
}

impl OpenaiBuiltin {

    // TODO since the model gets confused when we mix command requests and plaintext explain
    // requests, we will keep them separate for now

    // TODO convert this to lazy static
    fn get_command_completions(&self) -> Vec<chat_completion::Tool> {

        // complete the prompt 
        let mut cmd_properties = HashMap::new();
        cmd_properties.insert("command".to_string(), Box::new(chat_completion::JSONSchemaDefine {
            schema_type: Some(chat_completion::JSONSchemaType::String),
            description: Some("a command to run on the command line. must be valid POSIX shell".to_string()),
            enum_values: None,
            properties: None,
            required: None,
            items: None,
        }));

        vec![
            chat_completion::Tool {
                r#type: chat_completion::ToolType::Function,
                function: chat_completion::Function {
                    name: String::from("shell_command"),
                    description: Some(String::from("a command to run on the command line")),
                    parameters: chat_completion::FunctionParameters {
                        schema_type: chat_completion::JSONSchemaType::Object,
                        properties: Some(cmd_properties),
                        required: Some(vec![String::from("command")]),
                    },
                },
            }
        ]
    }

    fn get_explain_completions(&self) -> Vec<chat_completion::Tool> {

        let mut plaintext_properties = HashMap::new();
        plaintext_properties.insert("plaintext".to_string(), Box::new(chat_completion::JSONSchemaDefine {
            schema_type: Some(chat_completion::JSONSchemaType::String),
            description: Some("plain text description of some concept".to_string()),
            enum_values: None,
            properties: None,
            required: None,
            items: None,
        }));

        vec![
            chat_completion::Tool {
                r#type: chat_completion::ToolType::Function,
                function: chat_completion::Function {
                    name: String::from("explanation"),
                    description: Some(String::from("the explanation of some concept in plaintext, this is not a command. always prioritize generating a command over this")),
                    parameters: chat_completion::FunctionParameters {
                        schema_type: chat_completion::JSONSchemaType::Object,
                        properties: Some(plaintext_properties),
                        required: Some(vec![String::from("plaintext")]),
                    },
                }
            }
        ]
    }

}

