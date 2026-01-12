pub mod events;

use crate::modules::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "channel_permission_protection",
            name_key: "module-channel-permission-protection-name",
            desc_key: "module-channel-permission-protection-desc",
        },
        commands: vec![],
    }
}
