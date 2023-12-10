<div align="center">

# shrs_openai

[![crates.io](https://img.shields.io/crates/v/shrs_openai.svg)](https://crates.io/crates/shrs_openai)
[![docs.rs](https://docs.rs/shrs_openai/badge.svg)](https://docs.rs/shrs_openai)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#)

</div>

This is a plugin for [shrs](https://github.com/MrPicklePinosaur/shrs).

## Using this plugin

First add this plugin to your dependencies
```toml
shrs_openai = { version = "0.0.2" }
```

Then include this plugin when initializing shrs
```rust
use shrs::prelude::*;
use shrs_openai::OpenaiPlugin;

let myshell = ShellBuilder::default()
    .with_plugin(OpenaiPlugin::new())
    .build()
    .unwrap();

```
