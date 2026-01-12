pub mod events;

use super::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "role_protection",
            name_key: "module-role-protection-name",
            desc_key: "module-role-protection-desc",
        },
        commands: vec![],
    }
}

