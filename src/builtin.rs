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
    description: String,
    command: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Explanation {
    plaintext: String,
}

pub fn openai_builtin(
    mut openai: StateMut<OpenaiState>,
    mut content_queue: StateMut<PromptContentQueue>,
    args: &Vec<String>,
) -> anyhow::Result<CmdOutput> {

    if args.len() <= 1 {
        return Ok(CmdOutput::success());
    }

    // parse flags
    let mut flag_explain = false;
    let mut shift = 1;

    if args.len() >= 2 {
        if args[1] == "-e" {
            flag_explain = true;
            shift += 1;
        }
    }

    let args = &args[shift..].join(" ");
    // println!("args {args}");

    openai.chat_history.push(
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
            openai.chat_history.clone() // TODO this can be a pretty big copy
        )
        .tools(get_explain_completions())
        .tool_choice(chat_completion::ToolChoiceType::Auto)

    } else {

        ChatCompletionRequest::new(
            GPT3_5_TURBO_0613.to_string(),
            openai.chat_history.clone() // TODO this can be a pretty big copy
        )
        .tools(get_command_completions())
        .tool_choice(chat_completion::ToolChoiceType::Auto)
    };

    let result = openai.client.chat_completion(req)?;
    match result.choices[0].finish_reason {
        Some(chat_completion::FinishReason::tool_calls) => {


            let tool_calls = result.choices[0].message.tool_calls.as_ref().unwrap();
            for tool_call in tool_calls {
                let fn_name = tool_call.function.name.clone().unwrap();
                let arguments = tool_call.function.arguments.clone().unwrap();
                if fn_name == "shell_command" {

                    let cmd: Command = serde_json::from_str(&arguments)?;
                    println!("{}", cmd.description.bold());
                    // TODO could make auto-run configurable
                    content_queue.push(PromptContent { content: cmd.command, auto_run: false });

                } else if fn_name == "info_request" {
                } else if fn_name == "explanation" {

                    // apply some nice looking formatting
                    let explanation: Explanation = serde_json::from_str(&arguments)?;
                    let wrapped = textwrap::wrap(&explanation.plaintext, 80);
                    let formatted = wrapped.iter().map(|s| format!(" | {}", s.italic())).collect::<Vec<_>>().join("\n");
                    println!("{}", formatted);

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

// TODO since the model gets confused when we mix command requests and plaintext explain
// requests, we will keep them separate for now

// TODO convert this to lazy static
fn get_command_completions() -> Vec<chat_completion::Tool> {

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
    cmd_properties.insert("description".to_string(), Box::new(chat_completion::JSONSchemaDefine {
        schema_type: Some(chat_completion::JSONSchemaType::String),
        description: Some("description of what the command does".to_string()),
        enum_values: None,
        properties: None,
        required: None,
        items: None,
    }));

    let mut info_properties = HashMap::new();
    info_properties.insert("type".to_string(), Box::new(chat_completion::JSONSchemaDefine {
        schema_type: Some(chat_completion::JSONSchemaType::String),
        description: Some("the type of information to request from the yser".to_string()),
        enum_values: Some(vec!["history".into()]),
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
                    required: Some(vec![String::from("command"), String::from("description")]),
                },
            },
        },
        chat_completion::Tool {
            r#type: chat_completion::ToolType::Function,
            function: chat_completion::Function {
                name: String::from("info_request"),
                description: Some(String::from("if there is insufficient information, can ask for more from user")),
                parameters: chat_completion::FunctionParameters {
                    schema_type: chat_completion::JSONSchemaType::Object,
                    properties: Some(info_properties),
                    required: Some(vec![String::from("type")]),
                },
            },
        }
    ]
}

fn get_explain_completions() -> Vec<chat_completion::Tool> {

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


