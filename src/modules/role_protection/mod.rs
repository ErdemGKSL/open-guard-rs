pub mod events;

use super::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "role_protection",
            name_key: "module-role-protection-name",
            description_key: "module-role-protection-description",
        },
        commands: vec![],
    }
}
