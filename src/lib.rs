use shrs::prelude::*;

pub struct OpenaiPlugin;

impl OpenaiPlugin {
    pub fn new() -> Self {
        OpenaiPlugin
    }
}

impl Plugin for OpenaiPlugin {
    fn init(&self, shell: &mut ShellConfig) -> anyhow::Result<()> {
        Ok(())
    }
}
