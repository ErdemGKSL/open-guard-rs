pub mod events;

use crate::modules::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "channel_protection",
            name_key: "module-channel-protection-name",
            desc_key: "module-channel-protection-desc",
        },
        commands: vec![],
    }
}
