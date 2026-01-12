pub mod commands;
pub mod duration_parser;
pub mod events;

use crate::modules::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "moderation_protection",
            name_key: "module-moderation-protection-name",
            desc_key: "module-moderation-protection-desc",
        },
        commands: vec![commands::ban(), commands::kick(), commands::timeout()],
    }
}
