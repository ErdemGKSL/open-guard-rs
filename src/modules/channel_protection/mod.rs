pub mod commands;
pub mod events;

use crate::modules::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "channel_protection",
            name_key: "module-channel-protection-name",
            description_key: "module-channel-protection-desc",
        },
        commands: vec![commands::channel_protection()],
    }
}
