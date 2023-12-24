<div align="center">

# shrs_openai

[![crates.io](https://img.shields.io/crates/v/shrs_openai.svg)](https://crates.io/crates/shrs_openai)
[![docs.rs](https://docs.rs/shrs_openai/badge.svg)](https://docs.rs/shrs_openai)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#)

</div>

This is a plugin for [shrs](https://github.com/MrPicklePinosaur/shrs). It enables the ability to use ChatGPT to help you out on common command like tasks like:
- ask to write a command for you (list all stopped docker containers and delete them)
- answer questions based on your shell environment (why did my last command not work?)
- pass contents of files (why is this JSON file deformed?)

## Using this plugin

First add this plugin to your dependencies
```toml
shrs_openai = { version = "0.0.2" }
```

Then include this plugin when initializing shrs
```rust
use shrs::prelude::*;
use shrs_openai::OpenaiPlugin;

let api_key = std::env::var("OPENAI_KEY").unwrap().to_string();

let myshell = ShellBuilder::default()
    .with_plugin(OpenaiPlugin::new(api_key))
    .build()
    .unwrap();

```
