use shrs::prelude::*;
use shrs_openai::OpenaiPlugin;

fn main() {

    const OPENAI_KEY_ENV: &str = "OPENAI_KEY";

    let Ok(api_key) = std::env::var(OPENAI_KEY_ENV) else {
        eprintln!("Missing '{OPENAI_KEY_ENV}' environment variable");
        std::process::exit(1);
    };

    let myshell = ShellBuilder::default()
        .with_plugin(OpenaiPlugin::new(api_key))
        .build()
        .unwrap();

    myshell.run();
}
