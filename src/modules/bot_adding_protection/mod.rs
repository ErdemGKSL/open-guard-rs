pub mod events;

use crate::modules::{Module, ModuleDefinition};

pub const DEFINITION: ModuleDefinition = ModuleDefinition {
    id: "bot_adding_protection",
    name_key: "module-bot-adding-protection-name",
    description_key: "module-bot-adding-protection-description",
};

pub fn module() -> Module {
    Module {
        definition: DEFINITION,
        commands: vec![],
    }
}
