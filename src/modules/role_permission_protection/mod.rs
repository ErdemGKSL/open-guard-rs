pub mod events;

use super::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "role_permission_protection",
            name_key: "module-role-permission-protection-name",
            desc_key: "module-role-permission-protection-desc",
        },
        commands: vec![],
        event_handlers: vec![events::handler],
    }
}
