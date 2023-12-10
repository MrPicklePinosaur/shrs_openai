use shrs::prelude::*;
use shrs_openai::OpenaiPlugin;

fn main() {

    let api_key = std::env::var("OPENAI_KEY").unwrap().to_string();

    let myshell = ShellBuilder::default()
        .with_plugin(OpenaiPlugin::new(api_key))
        .build()
        .unwrap();

    myshell.run();
}
