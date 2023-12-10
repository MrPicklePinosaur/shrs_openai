use shrs::prelude::*;
use shrs_openai::OpenaiPlugin;

fn main() {
    let myshell = ShellBuilder::default()
        .with_plugin(OpenaiPlugin::new())
        .build()
        .unwrap();

    myshell.run();
}
